pub mod command;
pub mod light_command;
pub mod light_current;
pub mod light_encode_input;
pub mod lua;

use crate::actors::devices::handler::DeviceHandler;
use crate::decoding::DeviceRoleName;
use crate::device_command::{self, CommandTargets, DeviceCommandError, Outbound};
use crate::lua::LuaDecoder;
use crate::{
    device_registry::{Capability, Transport},
    event_bus::EventBusMessage,
    repo::intent::{DeviceKind, DeviceReport, IntentAttributes, IntentStatus},
    repo::light::{HistorySource, LightAttributes, LightSample, LightState},
    settings::IEEEAddress,
    state::AppState,
};
use light_command::LightCommand;
use light_current::LightCurrent;
use light_encode_input::LightEncodeInput;
use ractor::RpcReplyPort;
use uuid::Uuid;

const BRIGHTNESS_MAX: u64 = 254;

pub struct Entity {
    pub address: String,
    pub attributes: LightAttributes,
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
    pub traceparent: crate::tracing_context::TraceParent,
}

pub enum LightHandlerMessage {
    QueryPowerState {
        ieee_addr: IEEEAddress,
        traceparent: crate::tracing_context::TraceParent,
        reply: RpcReplyPort<bool>,
    },
    QueryState {
        ieee_addr: IEEEAddress,
        traceparent: crate::tracing_context::TraceParent,
        reply: RpcReplyPort<LightState>,
    },
    Set {
        ieee_addr: IEEEAddress,
        request: Box<SetRequest>,
        reply: RpcReplyPort<LightState>,
    },
    NewEvent(Box<NewEvent>),
    Reapply {
        ieee_addr: IEEEAddress,
        attributes: Box<LightAttributes>,
        traceparent: crate::tracing_context::TraceParent,
    },
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

impl crate::tracing_context::TracedMessage for LightHandlerMessage {
    fn traceparent(&self) -> Option<&str> {
        match self {
            LightHandlerMessage::NewEvent(event) => event.traceparent.as_deref(),
            LightHandlerMessage::QueryPowerState { traceparent, .. }
            | LightHandlerMessage::QueryState { traceparent, .. }
            | LightHandlerMessage::Reapply { traceparent, .. } => traceparent.as_deref(),
            _ => None,
        }
    }

    fn subject(&self) -> Option<&str> {
        match self {
            LightHandlerMessage::NewEvent(event) => Some(&event.entity.address),
            LightHandlerMessage::QueryPowerState { ieee_addr, .. }
            | LightHandlerMessage::QueryState { ieee_addr, .. }
            | LightHandlerMessage::Set { ieee_addr, .. }
            | LightHandlerMessage::Reapply { ieee_addr, .. }
            | LightHandlerMessage::TurnOn { ieee_addr }
            | LightHandlerMessage::TurnOff { ieee_addr }
            | LightHandlerMessage::Toggle { ieee_addr }
            | LightHandlerMessage::BrightnessMove { ieee_addr, .. }
            | LightHandlerMessage::ColourTemperatureMove { ieee_addr, .. }
            | LightHandlerMessage::SetColourTemperature { ieee_addr, .. }
            | LightHandlerMessage::SetBrightness { ieee_addr, .. }
            | LightHandlerMessage::SetColour { ieee_addr, .. } => Some(ieee_addr),
        }
    }
}

pub struct LightHandler {
    shared_actor_state: AppState,
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

pub fn normalise_hex(hex: &str) -> String {
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

    settle_confirmed_intents(shared_actor_state, &ieee_addr, &state).await;

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

async fn settle_confirmed_intents(
    shared_actor_state: &AppState,
    ieee_addr: &str,
    state: &LightState,
) {
    let repo = shared_actor_state.repos.intent();

    let pending = match repo.pending_for(DeviceKind::Light, ieee_addr).await {
        Ok(pending) => pending,
        Err(e) => {
            tracing::warn!("failed to read pending intents for {ieee_addr}: {e}");
            return;
        }
    };

    let report = DeviceReport::Light(state);

    let confirmed: Vec<i64> = pending
        .iter()
        .filter(|intent| intent.relative || intent.attributes.satisfied_by(&report))
        .map(|intent| intent.id)
        .collect();

    if confirmed.is_empty() {
        return;
    }

    tracing::info!(
        "confirmed {} intent(s) for {ieee_addr} from device report",
        confirmed.len()
    );

    if let Err(e) = repo.settle(&confirmed, IntentStatus::Confirmed).await {
        tracing::warn!("failed to settle intents for {ieee_addr}: {e}");
    }
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
        decoder: &LuaDecoder,
        ieee_addr: &str,
        request: SetRequest,
    ) -> Result<LightState, anyhow::Error> {
        let brightness = request.brightness.map(|brightness| {
            self.warn_if_unsupported(ieee_addr, Capability::Brightness);
            brightness.clamp(0, BRIGHTNESS_MAX)
        });

        let range = self
            .shared_actor_state
            .devices
            .device(ieee_addr)
            .and_then(|device| device.profile.as_ref())
            .and_then(|profile| profile.ranges.colour_temp);

        let colour_temp = request.colour_temp.and_then(|colour_temp| match range {
            Some(range) => Some(range.clamp(colour_temp)),
            None => {
                tracing::warn!("light {ieee_addr} does not support colour_temp, dropping it");
                None
            }
        });

        let colour = request.colour.map(|colour| {
            self.warn_if_unsupported(ieee_addr, Capability::Rgb);
            normalise_hex(&colour)
        });

        let attributes = LightAttributes {
            state: request
                .on
                .map(|on| if on { "ON" } else { "OFF" }.to_owned()),
            brightness: brightness.map(|brightness| brightness as i32),
            colour_temp: colour_temp.map(|colour_temp| colour_temp as i32),
            colour: colour.clone(),
        };

        if attributes.is_empty() {
            return self.stored_state(ieee_addr).await;
        }

        let command = LightCommand::Set {
            on: request.on,
            brightness,
            colour_temp,
            colour,
        };

        let sent = self.send(decoder, ieee_addr, &command).await?;

        if !sent {
            return self.stored_state(ieee_addr).await;
        }

        self.record_intent(ieee_addr, &attributes, false).await;

        self.stored_state(ieee_addr).await
    }

    async fn record_intent(&self, ieee_addr: &str, attributes: &LightAttributes, relative: bool) {
        if let Err(e) = self
            .shared_actor_state
            .repos
            .intent()
            .replace_pending(
                ieee_addr,
                &IntentAttributes::Light(attributes.clone()),
                relative,
                Uuid::new_v4(),
            )
            .await
        {
            tracing::warn!("failed to record intent for {ieee_addr}: {e}");
        }
    }

    async fn handle(
        &self,
        decoder: &LuaDecoder,
        message: LightHandlerMessage,
    ) -> Result<(), anyhow::Error> {
        match message {
            LightHandlerMessage::NewEvent(event) => {
                let Entity {
                    address,
                    attributes,
                } = event.entity;

                self.update_light_state(event.event_id, address, attributes)
                    .await?;
            }
            LightHandlerMessage::Reapply {
                ieee_addr,
                attributes,
                ..
            } => {
                self.send(decoder, &ieee_addr, &LightCommand::reapply(&attributes))
                    .await?;
            }
            LightHandlerMessage::TurnOn { ieee_addr } => {
                self.send_and_record(
                    decoder,
                    &ieee_addr,
                    &LightCommand::power(true),
                    &LightAttributes::state("ON"),
                )
                .await?;
            }
            LightHandlerMessage::TurnOff { ieee_addr } => {
                self.send_and_record(
                    decoder,
                    &ieee_addr,
                    &LightCommand::power(false),
                    &LightAttributes::state("OFF"),
                )
                .await?;
            }
            LightHandlerMessage::Toggle { ieee_addr } => {
                let on = self.stored_power_state(&ieee_addr).await?;
                let target = if on { "OFF" } else { "ON" };

                self.send_and_record(
                    decoder,
                    &ieee_addr,
                    &LightCommand::Toggle,
                    &LightAttributes::state(target),
                )
                .await?;
            }
            LightHandlerMessage::SetColourTemperature { ieee_addr, value } => {
                let request = SetRequest {
                    colour_temp: Some(value),
                    ..SetRequest::default()
                };

                self.apply_set(decoder, &ieee_addr, request).await?;
            }
            LightHandlerMessage::SetBrightness { ieee_addr, value } => {
                let request = SetRequest {
                    brightness: Some(value),
                    ..SetRequest::default()
                };

                self.apply_set(decoder, &ieee_addr, request).await?;
            }
            LightHandlerMessage::SetColour { ieee_addr, hex } => {
                let request = SetRequest {
                    colour: Some(hex),
                    ..SetRequest::default()
                };

                self.apply_set(decoder, &ieee_addr, request).await?;
            }
            LightHandlerMessage::BrightnessMove {
                ieee_addr,
                value,
                on_off,
            } => {
                self.warn_if_unsupported(&ieee_addr, Capability::Brightness);

                let command = LightCommand::BrightnessMove { value, on_off };

                if self.send(decoder, &ieee_addr, &command).await? {
                    self.record_intent(&ieee_addr, &LightAttributes::default(), true)
                        .await;
                }
            }
            LightHandlerMessage::ColourTemperatureMove { ieee_addr, value } => {
                self.warn_if_unsupported(&ieee_addr, Capability::ColourTemp);

                let command = LightCommand::ColourTempMove { value };

                if self.send(decoder, &ieee_addr, &command).await? {
                    self.record_intent(&ieee_addr, &LightAttributes::default(), true)
                        .await;
                }
            }
            LightHandlerMessage::Set {
                ieee_addr,
                request,
                reply,
            } => {
                let state = self.apply_set(decoder, &ieee_addr, *request).await?;

                reply.send(state)?;
            }
            LightHandlerMessage::QueryState {
                ieee_addr, reply, ..
            } => {
                let state = self.stored_state(&ieee_addr).await?;

                reply.send(state)?;
            }
            LightHandlerMessage::QueryPowerState {
                ieee_addr, reply, ..
            } => {
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

    async fn send_and_record(
        &self,
        decoder: &LuaDecoder,
        ieee_addr: &str,
        command: &LightCommand,
        attributes: &LightAttributes,
    ) -> Result<(), anyhow::Error> {
        if self.send(decoder, ieee_addr, command).await? {
            self.record_intent(ieee_addr, attributes, false).await;
        }

        Ok(())
    }

    async fn send(
        &self,
        decoder: &LuaDecoder,
        ieee_addr: &str,
        command: &LightCommand,
    ) -> Result<bool, anyhow::Error> {
        let devices = &self.shared_actor_state.devices;

        let Some(profile) = devices
            .device(ieee_addr)
            .and_then(|device| device.profile.as_ref())
        else {
            tracing::warn!("not sending light command to {ieee_addr}: no model");
            return Ok(false);
        };

        let role = if profile.roles.contains(&DeviceRoleName::Light) {
            DeviceRoleName::Light
        } else if profile.roles.contains(&DeviceRoleName::SmartSwitch) {
            DeviceRoleName::SmartSwitch
        } else {
            tracing::warn!("not sending light command to {ieee_addr}: it is not a light or switch");
            return Ok(false);
        };

        let current = self.stored_state(ieee_addr).await?;
        let input = LightEncodeInput {
            command,
            current: LightCurrent::from(&current),
        };

        let payload = match profile.encode(decoder, role, &input) {
            Ok(Some(payload)) => payload,
            Ok(None) => {
                tracing::warn!("light {ieee_addr} does not support {command:?}");
                return Ok(false);
            }
            Err(e) => {
                tracing::warn!("failed to encode {command:?} for light {ieee_addr}: {e}");
                return Ok(false);
            }
        };

        let targets = CommandTargets::new(devices, &self.shared_actor_state.handles);

        match device_command::send(&targets, ieee_addr, role, Outbound::Json(payload)).await {
            Ok(()) => Ok(true),
            Err(DeviceCommandError::Mqtt(e)) => Err(e.into()),
            Err(e) => {
                tracing::warn!("not sending light command to {ieee_addr}: {e}");
                Ok(false)
            }
        }
    }
}

impl DeviceHandler for LightHandler {
    const NAME: &'static str = LightHandler::NAME;

    type Message = LightHandlerMessage;
    type State = LuaDecoder;

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    const ROLE: DeviceRoleName = DeviceRoleName::Light;

    fn init_state(&self) -> anyhow::Result<Self::State> {
        let settings = &self.shared_actor_state.settings;

        Ok(settings
            .model_sources
            .decoder(Transport::Mqtt, &settings.lua)?)
    }

    async fn handle(
        &self,
        message: Self::Message,
        decoder: &mut Self::State,
    ) -> anyhow::Result<()> {
        Self::handle(self, decoder, message).await
    }
}

#[cfg(test)]
mod tests {
    use super::colour_hex;

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
}
