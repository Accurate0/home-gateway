use crate::{
    device_registry::Capability,
    esphome::light_command_topic,
    event_bus::EventBusMessage,
    mqtt::ZIGBEE2MQTT_BASE,
    settings::IEEEAddress,
    types::SharedActorState,
    zigbee2mqtt::{
        Aqara_T1,
        IKEA_LED2201G8::{self},
        Phillips_9290012573A,
    },
};
use ractor::{
    ActorProcessingErr, ActorRef, RpcReplyPort,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub enum Entity {
    Phillips9290012573A(Phillips_9290012573A::Phillips9290012573A),
    IKEALED2201G8(IKEA_LED2201G8::IKEALED2201G8),
    AqaraT1(Aqara_T1::AqaraT1),
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
}

pub enum LightHandlerMessage {
    QueryPowerState {
        ieee_addr: IEEEAddress,
        reply: RpcReplyPort<bool>,
    },
    NewEvent(Box<NewEvent>),
    TurnOn {
        ieee_addr: IEEEAddress,
    },
    TurnOff {
        ieee_addr: IEEEAddress,
    },
    Toggle {
        ieee_addr: IEEEAddress,
    },
    BrightnessMove {
        ieee_addr: IEEEAddress,
        value: i64,
        on_off: bool,
    },
    ColourTemperatureMove {
        ieee_addr: IEEEAddress,
        value: i64,
    },
    SetBrightness {
        ieee_addr: IEEEAddress,
        value: u64,
    },
    SetColour {
        ieee_addr: IEEEAddress,
        hex: String,
    },
}

pub struct LightHandler {
    shared_actor_state: SharedActorState,
}

fn esphome_command(state: &serde_json::Value) -> Option<serde_json::Value> {
    let object = state.as_object()?;
    let mut out = serde_json::Map::new();

    for (key, value) in object {
        match key.as_str() {
            "state" => match value.as_str()? {
                on @ ("ON" | "OFF") => {
                    out.insert("state".into(), on.into());
                }
                _ => return None,
            },
            "brightness" => {
                let scaled = (value.as_u64()? * 255).div_ceil(254);
                out.insert("brightness".into(), scaled.into());
            }
            "color" => {
                let hex = value.get("hex")?.as_str()?.trim_start_matches('#');
                let rgb = u32::from_str_radix(hex, 16).ok()?;
                out.insert(
                    "color".into(),
                    serde_json::json!({
                        "r": (rgb >> 16) & 0xff,
                        "g": (rgb >> 8) & 0xff,
                        "b": rgb & 0xff,
                    }),
                );
            }
            _ => return None,
        }
    }

    Some(out.into())
}

pub async fn record_light_state(
    shared_actor_state: &SharedActorState,
    event_id: Uuid,
    ieee_addr: IEEEAddress,
    state: String,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        "INSERT INTO light_state (ieee_address, state) VALUES ($1, $2) ON CONFLICT (ieee_address) DO UPDATE SET state = EXCLUDED.state",
        ieee_addr,
        state,
    ).execute(&shared_actor_state.db).await?;

    shared_actor_state
        .event_bus
        .publish(EventBusMessage::Light {
            event_id,
            on: state == "ON",
            ieee_addr,
        });

    Ok(())
}

impl LightHandler {
    pub const NAME: &str = "light";

    async fn update_light_state(
        &self,
        event_id: Uuid,
        ieee_addr: IEEEAddress,
        state: String,
    ) -> Result<(), anyhow::Error> {
        record_light_state(&self.shared_actor_state, event_id, ieee_addr, state).await
    }

    async fn stored_power_state(&self, ieee_addr: &str) -> Result<bool, anyhow::Error> {
        let light_state = sqlx::query!(
            "SELECT state FROM light_state WHERE ieee_address = $1",
            ieee_addr
        )
        .fetch_optional(&self.shared_actor_state.db)
        .await?;

        Ok(light_state.is_some_and(|row| row.state == "ON"))
    }

    fn warn_if_unsupported(&self, ieee_addr: &str, capability: Capability) {
        if !self
            .shared_actor_state
            .devices
            .capabilities(ieee_addr)
            .contains(&capability)
        {
            tracing::warn!("light {ieee_addr} does not support {capability:?}");
        }
    }

    async fn handle(&self, message: LightHandlerMessage) -> Result<(), anyhow::Error> {
        match message {
            LightHandlerMessage::NewEvent(event) => {
                let event_id = event.event_id;
                match event.entity {
                    Entity::Phillips9290012573A(phillips_9290012573_a) => {
                        self.update_light_state(
                            event_id,
                            phillips_9290012573_a.device.ieee_addr,
                            phillips_9290012573_a.state,
                        )
                        .await?
                    }
                    Entity::IKEALED2201G8(ikealed2201_g8) => {
                        self.update_light_state(
                            event_id,
                            ikealed2201_g8.device.ieee_addr,
                            ikealed2201_g8.state,
                        )
                        .await?
                    }
                    Entity::AqaraT1(aqara_t1) => {
                        self.update_light_state(event_id, aqara_t1.device.ieee_addr, aqara_t1.state)
                            .await?
                    }
                }
            }
            LightHandlerMessage::TurnOn { ieee_addr } => {
                self.send_mqtt_state(ieee_addr, serde_json::json!({"state": "ON"}))
                    .await?;
            }
            LightHandlerMessage::TurnOff { ieee_addr } => {
                self.send_mqtt_state(ieee_addr, serde_json::json!({"state": "OFF"}))
                    .await?;
            }
            LightHandlerMessage::Toggle { ieee_addr } => {
                let state = if self
                    .shared_actor_state
                    .devices
                    .esphome_light(&ieee_addr)
                    .is_some()
                {
                    let on = self.stored_power_state(&ieee_addr).await?;
                    serde_json::json!({"state": if on { "OFF" } else { "ON" }})
                } else {
                    serde_json::json!({"state": "TOGGLE"})
                };

                self.send_mqtt_state(ieee_addr, state).await?;
            }
            LightHandlerMessage::SetBrightness { ieee_addr, value } => {
                self.warn_if_unsupported(&ieee_addr, Capability::Brightness);
                let value = value.clamp(0, 254);

                self.send_mqtt_state(ieee_addr, serde_json::json!({"brightness": value}))
                    .await?;
            }
            LightHandlerMessage::SetColour { ieee_addr, hex } => {
                self.warn_if_unsupported(&ieee_addr, Capability::Rgb);
                self.send_mqtt_state(ieee_addr, serde_json::json!({"color": {"hex": hex}}))
                    .await?;
            }
            LightHandlerMessage::BrightnessMove {
                ieee_addr,
                value,
                on_off,
            } => {
                self.warn_if_unsupported(&ieee_addr, Capability::Brightness);
                if on_off {
                    self.send_mqtt_state(
                        ieee_addr,
                        serde_json::json!({"brightness_move_onoff": value}),
                    )
                    .await?;
                } else {
                    self.send_mqtt_state(ieee_addr, serde_json::json!({"brightness_move": value}))
                        .await?;
                }
            }
            LightHandlerMessage::ColourTemperatureMove { ieee_addr, value } => {
                self.warn_if_unsupported(&ieee_addr, Capability::ColourTemp);
                let state = if value == 0 {
                    serde_json::json!({"color_temp_move": "stop"})
                } else {
                    serde_json::json!({"color_temp_move": value})
                };

                self.send_mqtt_state(ieee_addr, state).await?;
            }
            LightHandlerMessage::QueryPowerState { ieee_addr, reply } => {
                let light_state = sqlx::query!(
                    "SELECT state FROM light_state WHERE ieee_address = $1",
                    ieee_addr
                )
                .fetch_one(&self.shared_actor_state.db)
                .await?;

                let is_on = light_state.state == "ON";

                reply.send(is_on)?;
            }
        }

        Ok(())
    }

    async fn send_mqtt_state(
        &self,
        ieee_addr: String,
        state: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        if let Some(object_id) = self.shared_actor_state.devices.esphome_light(&ieee_addr) {
            let topic = light_command_topic(&ieee_addr, object_id);
            let Some(state) = esphome_command(&state) else {
                tracing::warn!("esphome light {ieee_addr} does not support command: {state}");
                return Ok(());
            };
            self.shared_actor_state
                .mqtt
                .send_event(topic, state)
                .await?;
            return Ok(());
        }

        let target = self
            .shared_actor_state
            .devices
            .friendly_name(&ieee_addr)
            .await
            .unwrap_or_else(|| ieee_addr.clone());

        let topic = format!("{ZIGBEE2MQTT_BASE}/{target}/set");
        self.shared_actor_state
            .mqtt
            .send_event(topic, state)
            .await?;

        Ok(())
    }
}

impl Worker for LightHandler {
    type Key = ();
    type Message = LightHandlerMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), LightHandlerMessage>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), LightHandlerMessage>>,
        Job { msg, .. }: Job<(), LightHandlerMessage>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg).await {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct LightHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}
impl WorkerBuilder<LightHandler, ()> for LightHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (LightHandler, ()) {
        (
            LightHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::esphome_command;

    #[test]
    fn translates_on_off_brightness_and_colour() {
        assert_eq!(
            esphome_command(&serde_json::json!({"state": "ON"})),
            Some(serde_json::json!({"state": "ON"}))
        );
        assert_eq!(
            esphome_command(&serde_json::json!({"brightness": 254})),
            Some(serde_json::json!({"brightness": 255}))
        );
        assert_eq!(
            esphome_command(&serde_json::json!({"brightness": 0})),
            Some(serde_json::json!({"brightness": 0}))
        );
        assert_eq!(
            esphome_command(&serde_json::json!({"color": {"hex": "#ff8000"}})),
            Some(serde_json::json!({"color": {"r": 255, "g": 128, "b": 0}}))
        );
    }

    #[test]
    fn rejects_zigbee_only_commands() {
        for command in [
            serde_json::json!({"state": "TOGGLE"}),
            serde_json::json!({"brightness_move": 40}),
            serde_json::json!({"brightness_move_onoff": 40}),
            serde_json::json!({"color_temp_move": "stop"}),
        ] {
            assert_eq!(esphome_command(&command), None, "{command}");
        }
    }
}
