use crate::{
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
    #[allow(unused)]
    pub event_id: Uuid,
    pub entity: Entity,
}

pub enum LightHandlerMessage {
    QueryPowerState {
        ieee_addr: IEEEAddress,
        reply: RpcReplyPort<bool>,
    },
    NewEvent(NewEvent),
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
    #[allow(unused)]
    SetBrightness {
        ieee_addr: IEEEAddress,
        value: u64,
    },
}

pub struct LightHandler {
    shared_actor_state: SharedActorState,
}

impl LightHandler {
    pub const NAME: &str = "light";

    async fn update_light_state(
        &self,
        ieee_addr: IEEEAddress,
        state: String,
    ) -> Result<(), anyhow::Error> {
        sqlx::query!(
            "INSERT INTO light_state (ieee_address, state) VALUES ($1, $2) ON CONFLICT (ieee_address) DO UPDATE SET state = EXCLUDED.state",
            ieee_addr,
            state,
        ).execute(&self.shared_actor_state.db).await?;

        Ok(())
    }

    async fn handle(&self, message: LightHandlerMessage) -> Result<(), anyhow::Error> {
        match message {
            LightHandlerMessage::NewEvent(event) => match event.entity {
                Entity::Phillips9290012573A(phillips_9290012573_a) => {
                    self.update_light_state(
                        phillips_9290012573_a.device.ieee_addr,
                        phillips_9290012573_a.state,
                    )
                    .await?
                }
                Entity::IKEALED2201G8(ikealed2201_g8) => {
                    self.update_light_state(ikealed2201_g8.device.ieee_addr, ikealed2201_g8.state)
                        .await?
                }
                Entity::AqaraT1(aqara_t1) => {
                    self.update_light_state(aqara_t1.device.ieee_addr, aqara_t1.state)
                        .await?
                }
            },
            LightHandlerMessage::TurnOn { ieee_addr } => {
                self.send_mqtt_state(ieee_addr, serde_json::json!({"state": "ON"}))
                    .await?;
            }
            LightHandlerMessage::TurnOff { ieee_addr } => {
                self.send_mqtt_state(ieee_addr, serde_json::json!({"state": "OFF"}))
                    .await?;
            }
            LightHandlerMessage::Toggle { ieee_addr } => {
                self.send_mqtt_state(ieee_addr, serde_json::json!({"state": "TOGGLE"}))
                    .await?;
            }
            LightHandlerMessage::SetBrightness { ieee_addr, value } => {
                let value = if value >= 254 {
                    254
                } else if value <= 0 {
                    0
                } else {
                    value
                };

                self.send_mqtt_state(ieee_addr, serde_json::json!({"brightness": value}))
                    .await?;
            }
            LightHandlerMessage::BrightnessMove {
                ieee_addr,
                value,
                on_off,
            } => {
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
        let friendly_name = {
            let devices_map = self.shared_actor_state.known_devices_map.read().await;
            devices_map.get(&ieee_addr).cloned()
        };

        let Some(friendly_name) = friendly_name else {
            tracing::warn!("could not find device for {ieee_addr}");
            return Ok(());
        };

        let topic = format!("{ZIGBEE2MQTT_BASE}/{friendly_name}/set");
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
