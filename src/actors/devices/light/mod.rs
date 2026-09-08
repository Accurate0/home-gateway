use crate::actors::devices::handler::DeviceHandler;
use crate::integrations::mqtt::MqttClient;
use crate::{
    device_registry::Capability,
    event_bus::EventBusMessage,
    integrations::esphome::light_command_topic,
    integrations::mqtt::ZIGBEE2MQTT_BASE,
    repo::light::{HistorySource, LightAttributes, LightSample, LightState},
    settings::IEEEAddress,
    state::AppState,
};
use ractor::RpcReplyPort;
use uuid::Uuid;

const COLOUR_TEMP_MIN_MIREDS: u64 = 153;
const COLOUR_TEMP_MAX_MIREDS: u64 = 500;
const BRIGHTNESS_MAX: u64 = 254;

pub enum Entity {
    Zigbee {
        address: String,
        attributes: LightAttributes,
    },
    Esphome {
        node: String,
        attributes: LightAttributes,
    },
}

#[derive(Debug, Clone, Default)]
pub struct SetRequest {
    pub on: Option<bool>,
    pub brightness: Option<u64>,
    pub colour_temp: Option<u64>,
    pub colour: Option<String>,
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
    QueryState {
        ieee_addr: IEEEAddress,
        reply: RpcReplyPort<LightState>,
    },
    Set {
        ieee_addr: IEEEAddress,
        request: Box<SetRequest>,
        reply: RpcReplyPort<LightState>,
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
    SetColourTemperature {
        ieee_addr: IEEEAddress,
        value: u64,
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
    shared_actor_state: AppState,
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

pub fn colour_hex(value: &serde_json::Value) -> Option<String> {
    let object = value.as_object()?;

    if let Some(hex) = object.get("hex").and_then(serde_json::Value::as_str) {
        return Some(normalise_hex(hex));
    }

    let x = object.get("x")?.as_f64()?;
    let y = object.get("y")?.as_f64()?;

    (y > 0.0).then(|| xy_to_hex(x, y))
}

fn normalise_hex(hex: &str) -> String {
    format!("#{}", hex.trim_start_matches('#').to_lowercase())
}

fn xy_to_hex(x: f64, y: f64) -> String {
    let big_x = x / y;
    let big_z = (1.0 - x - y) / y;

    let r = big_x * 3.2406 - 1.5372 + big_z * -0.4986;
    let g = big_x * -0.9689 + 1.8758 + big_z * 0.0415;
    let b = big_x * 0.0557 - 0.2040 + big_z * 1.0570;

    let max = r.max(g).max(b).max(1.0);
    let channel = |value: f64| {
        let normalised = (value / max).clamp(0.0, 1.0);
        let gamma = if normalised <= 0.0031308 {
            12.92 * normalised
        } else {
            1.055 * normalised.powf(1.0 / 2.4) - 0.055
        };

        (gamma * 255.0).round() as u8
    };

    format!("#{:02x}{:02x}{:02x}", channel(r), channel(g), channel(b))
}

async fn record_light_state(
    shared_actor_state: &AppState,
    event_id: Uuid,
    ieee_addr: IEEEAddress,
    attributes: LightAttributes,
) -> Result<LightState, anyhow::Error> {
    let (state, previous) = shared_actor_state
        .repos
        .light()
        .upsert_state_returning_previous(&ieee_addr, &attributes)
        .await?;

    if previous.as_ref() != Some(&state) {
        let sample = LightSample {
            address: ieee_addr.clone(),
            device_id: shared_actor_state
                .devices
                .id_for_address(&ieee_addr)
                .map(str::to_owned),
            source: HistorySource::Edge,
            event_id: Some(event_id),
            state: state.clone(),
        };

        if let Err(e) = shared_actor_state
            .repos
            .light()
            .record_history(sample)
            .await
        {
            tracing::warn!("failed to record light history for {ieee_addr}: {e}");
        }
    }

    shared_actor_state
        .event_bus
        .publish(EventBusMessage::Light {
            event_id,
            on: state.on,
            brightness: state.brightness,
            colour_temp: state.colour_temp,
            colour: state.colour.clone(),
            ieee_addr,
        });

    Ok(state)
}

impl LightHandler {
    pub const NAME: &str = "light";

    async fn update_light_state(
        &self,
        event_id: Uuid,
        ieee_addr: IEEEAddress,
        attributes: LightAttributes,
    ) -> Result<LightState, anyhow::Error> {
        record_light_state(&self.shared_actor_state, event_id, ieee_addr, attributes).await
    }

    async fn stored_state(&self, ieee_addr: &str) -> Result<LightState, anyhow::Error> {
        let state = self.shared_actor_state.repos.light().get(ieee_addr).await?;

        Ok(state.unwrap_or(LightState {
            on: false,
            brightness: None,
            colour_temp: None,
            colour: None,
        }))
    }

    async fn stored_power_state(&self, ieee_addr: &str) -> Result<bool, anyhow::Error> {
        let is_on = self
            .shared_actor_state
            .repos
            .light()
            .is_on(ieee_addr)
            .await?;

        Ok(is_on.unwrap_or(false))
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

    async fn apply_set(
        &self,
        ieee_addr: &str,
        request: SetRequest,
    ) -> Result<LightState, anyhow::Error> {
        let mut payload = serde_json::Map::new();
        let mut attributes = LightAttributes::default();

        if let Some(on) = request.on {
            let state = if on { "ON" } else { "OFF" };
            payload.insert("state".into(), state.into());
            attributes.state = Some(state.to_owned());
        }

        if let Some(brightness) = request.brightness {
            self.warn_if_unsupported(ieee_addr, Capability::Brightness);
            let brightness = brightness.clamp(0, BRIGHTNESS_MAX);
            payload.insert("brightness".into(), brightness.into());
            attributes.brightness = Some(brightness as i32);
        }

        if let Some(colour_temp) = request.colour_temp {
            self.warn_if_unsupported(ieee_addr, Capability::ColourTemp);
            let colour_temp = colour_temp.clamp(COLOUR_TEMP_MIN_MIREDS, COLOUR_TEMP_MAX_MIREDS);
            payload.insert("color_temp".into(), colour_temp.into());
            attributes.colour_temp = Some(colour_temp as i32);
        }

        if let Some(colour) = request.colour {
            self.warn_if_unsupported(ieee_addr, Capability::Rgb);
            let colour = normalise_hex(&colour);
            payload.insert("color".into(), serde_json::json!({"hex": colour}));
            attributes.colour = Some(colour);
        }

        if attributes.is_empty() {
            return self.stored_state(ieee_addr).await;
        }

        let sent = self
            .send_mqtt_state(ieee_addr.to_owned(), payload.into())
            .await?;

        if !sent {
            return self.stored_state(ieee_addr).await;
        }

        self.update_light_state(Uuid::new_v4(), ieee_addr.to_owned(), attributes)
            .await
    }

    async fn handle(&self, message: LightHandlerMessage) -> Result<(), anyhow::Error> {
        match message {
            LightHandlerMessage::NewEvent(event) => {
                let event_id = event.event_id;
                match event.entity {
                    Entity::Zigbee {
                        address,
                        attributes,
                    } => {
                        self.update_light_state(event_id, address, attributes)
                            .await?;
                    }
                    Entity::Esphome { node, attributes } => {
                        self.update_light_state(event_id, node, attributes).await?;
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
            LightHandlerMessage::SetColourTemperature { ieee_addr, value } => {
                self.warn_if_unsupported(&ieee_addr, Capability::ColourTemp);
                let value = value.clamp(COLOUR_TEMP_MIN_MIREDS, COLOUR_TEMP_MAX_MIREDS);

                self.send_mqtt_state(ieee_addr, serde_json::json!({"color_temp": value}))
                    .await?;
            }
            LightHandlerMessage::SetBrightness { ieee_addr, value } => {
                self.warn_if_unsupported(&ieee_addr, Capability::Brightness);
                let value = value.clamp(0, BRIGHTNESS_MAX);

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
            LightHandlerMessage::Set {
                ieee_addr,
                request,
                reply,
            } => {
                let state = self.apply_set(&ieee_addr, *request).await?;

                reply.send(state)?;
            }
            LightHandlerMessage::QueryState { ieee_addr, reply } => {
                let state = self.stored_state(&ieee_addr).await?;

                reply.send(state)?;
            }
            LightHandlerMessage::QueryPowerState { ieee_addr, reply } => {
                let is_on = self
                    .shared_actor_state
                    .repos
                    .light()
                    .is_on(&ieee_addr)
                    .await?
                    .unwrap_or(false);

                reply.send(is_on)?;
            }
        }

        Ok(())
    }

    async fn send_mqtt_state(
        &self,
        ieee_addr: String,
        state: serde_json::Value,
    ) -> Result<bool, anyhow::Error> {
        if let Some(object_id) = self.shared_actor_state.devices.esphome_light(&ieee_addr) {
            let topic = light_command_topic(&ieee_addr, object_id);
            let Some(state) = esphome_command(&state) else {
                tracing::warn!("esphome light {ieee_addr} does not support command: {state}");
                return Ok(false);
            };
            self.shared_actor_state
                .handles
                .expect::<MqttClient>()
                .send_event(topic, state)
                .await?;
            return Ok(true);
        }

        let target = self
            .shared_actor_state
            .devices
            .friendly_name(&ieee_addr)
            .await
            .unwrap_or_else(|| ieee_addr.clone());

        let topic = format!("{ZIGBEE2MQTT_BASE}/{target}/set");
        self.shared_actor_state
            .handles
            .expect::<MqttClient>()
            .send_event(topic, state)
            .await?;

        Ok(true)
    }
}

impl DeviceHandler for LightHandler {
    const NAME: &'static str = LightHandler::NAME;
    const WORKERS: usize = 1;

    type Message = LightHandlerMessage;
    type State = ();

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    async fn handle(&self, message: Self::Message, _state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message).await
    }
}

#[cfg(test)]
mod tests {
    use super::{colour_hex, esphome_command};

    #[test]
    fn reads_colour_from_xy_and_hex() {
        assert_eq!(
            colour_hex(&serde_json::json!({"hex": "#FF8000"})),
            Some("#ff8000".to_owned())
        );
        assert_eq!(
            colour_hex(&serde_json::json!({"x": 0.3127, "y": 0.3290})),
            Some("#ffffff".to_owned())
        );
        assert_eq!(colour_hex(&serde_json::json!({"x": 0.7, "y": 0.0})), None);
        assert_eq!(colour_hex(&serde_json::json!("red")), None);
    }

    #[test]
    fn xy_red_and_blue_primaries_map_to_their_channel() {
        let red = colour_hex(&serde_json::json!({"x": 0.64, "y": 0.33})).expect("red");
        let blue = colour_hex(&serde_json::json!({"x": 0.15, "y": 0.06})).expect("blue");

        assert!(red.starts_with("#ff"), "{red}");
        assert!(blue.ends_with("ff"), "{blue}");
    }

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
