mod air_purifier_command_error;
mod air_purifier_reading;
pub mod command;
mod command_request;
pub mod lua;

pub use air_purifier_command_error::AirPurifierCommandError;
pub use air_purifier_reading::AirPurifierReading;
pub use command_request::CommandRequest;

use serde_json::json;
use uuid::Uuid;

use crate::actors::devices::handler::DeviceHandler;
use crate::decoding::{Decoders, DeviceRoleName};
use crate::device_command::{self, CommandOutcome, CommandTargets, Outbound};
use crate::event_bus::EventBusMessage;
use crate::repo::air_purifier::AirPurifierRecord;
use crate::state::AppState;
use crate::workflows::definition::AirPurifierCommand;

pub struct NewEvent {
    pub event_id: Uuid,
    pub reading: AirPurifierReading,
    pub traceparent: crate::telemetry::context::TraceParent,
}

pub enum Message {
    NewEvent(NewEvent),
    Command(CommandRequest),
}

impl crate::telemetry::context::TracedMessage for Message {
    fn traceparent(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => event.traceparent.as_deref(),
            Message::Command(request) => request.traceparent.as_deref(),
        }
    }

    fn subject(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => Some(&event.reading.address),
            Message::Command(request) => Some(&request.address),
        }
    }
}

pub struct AirPurifierHandler {
    shared_actor_state: AppState,
}

impl AirPurifierHandler {
    pub const NAME: &str = "air_purifier";

    async fn handle(&self, decoders: &Decoders, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::NewEvent(NewEvent {
                event_id, reading, ..
            }) => self.record(event_id, reading).await,
            Message::Command(CommandRequest {
                address,
                command,
                reply,
                ..
            }) => {
                let outcome = self.command(decoders, &address, command).await;

                match &outcome {
                    Ok(outcome) => tracing::info!("air purifier {address} {command}: {outcome}"),
                    Err(e) => tracing::warn!("air purifier {address} {command} failed: {e}"),
                }

                if reply.send(outcome).is_err() {
                    tracing::debug!("air purifier {address} {command} caller went away");
                }

                Ok(())
            }
        }
    }

    async fn command(
        &self,
        decoders: &Decoders,
        address: &str,
        command: AirPurifierCommand,
    ) -> Result<CommandOutcome, String> {
        let devices = &self.shared_actor_state.devices;

        let Some(settings) = devices.air_purifier(address) else {
            return Err(format!("{address} is not an air purifier"));
        };

        let Some(device) = devices.device(address) else {
            return Err(format!("{address} is not a registered device"));
        };

        let Some(profile) = device.profile.as_ref() else {
            return Err(format!("{address} has no model"));
        };

        let Some(decoder) = decoders.get(device.transport) else {
            return Err(format!("no `{}` models are loaded", device.transport));
        };

        let current = self
            .shared_actor_state
            .repos
            .air_purifier()
            .latest(&settings.id)
            .await
            .map_err(|e| format!("failed to load the current state: {e}"))?
            .map(|row| {
                json!({
                    "on": row.is_on,
                    "mode": row.mode,
                    "speed": row.speed,
                    "display": row.display,
                })
            })
            .unwrap_or_else(|| json!({}));

        let input = json!({
            "command": command,
            "current": current,
        });

        let payload = profile
            .encode::<_, serde_json::Value>(decoder, DeviceRoleName::AirPurifier, &input)
            .map_err(|e| format!("failed to encode {command}: {e}"))?;

        let Some(payload) = payload else {
            return Ok(CommandOutcome::Unchanged);
        };

        let targets = CommandTargets::new(devices, &self.shared_actor_state.handles);

        device_command::send(
            &targets,
            address,
            DeviceRoleName::AirPurifier,
            Outbound::Json(payload),
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(CommandOutcome::Sent)
    }

    async fn record(
        &self,
        event_id: Uuid,
        reading: AirPurifierReading,
    ) -> Result<(), anyhow::Error> {
        let AirPurifierReading {
            address,
            on,
            mode,
            speed,
            pm25,
            filter_life,
            display,
        } = reading;

        let Some(settings) = self.shared_actor_state.devices.air_purifier(&address) else {
            tracing::warn!("ignoring air purifier reading for undeclared device {address}");
            return Ok(());
        };

        let previous = self
            .shared_actor_state
            .repos
            .air_purifier()
            .record(&AirPurifierRecord {
                event_id,
                device_id: &settings.id,
                is_on: on,
                mode,
                speed,
                pm25,
                filter_life,
                display,
            })
            .await?;

        let mode = mode.or(previous.as_ref().and_then(|row| row.mode));
        let speed = speed.or(previous.as_ref().and_then(|row| row.speed));

        let display = display.or(previous.as_ref().and_then(|row| row.display));

        let unchanged = previous.as_ref().is_some_and(|row| {
            row.is_on == on && row.mode == mode && row.speed == speed && row.display == display
        });

        if unchanged {
            tracing::trace!("air purifier {} is unchanged", settings.id);
            return Ok(());
        }

        tracing::info!(
            "air purifier {} is {} (mode {mode:?}, speed {speed:?})",
            settings.id,
            if on { "on" } else { "off" }
        );

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::AirPurifier {
                event_id,
                device_id: settings.id.clone(),
                name: settings.name.clone(),
                on,
                mode,
                speed,
                display,
            });

        Ok(())
    }
}

impl DeviceHandler for AirPurifierHandler {
    const NAME: &'static str = AirPurifierHandler::NAME;

    type Message = Message;
    type State = Decoders;

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    const ROLE: DeviceRoleName = DeviceRoleName::AirPurifier;

    fn init_state(&self) -> anyhow::Result<Self::State> {
        let settings = &self.shared_actor_state.settings;

        Ok(settings.model_sources.decoders(&settings.lua)?)
    }

    async fn handle(
        &self,
        message: Self::Message,
        decoders: &mut Self::State,
    ) -> anyhow::Result<()> {
        Self::handle(self, decoders, message).await
    }
}
