pub mod command;
mod command_request;
mod garage_door_command_error;
mod garage_door_reading;
pub mod lua;

pub use command_request::CommandRequest;
pub use garage_door_command_error::GarageDoorCommandError;
pub use garage_door_reading::GarageDoorReading;

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::actors::devices::handler::DeviceHandler;
use crate::decoding::{Decoders, DeviceRoleName};
use crate::device_command::{self, CommandOutcome, CommandTargets, Outbound};
use crate::event_bus::EventBusMessage;
use crate::notify::{Notification, notify};
use crate::repo::garage_door::GarageDoorState;
use crate::settings::{ArmedDoorStates, GarageDoorSettings, NotificationSource, NotifyCategory};
use crate::state::AppState;
use crate::workflows::definition::GarageDoorCommand;

pub struct NewEvent {
    pub event_id: Uuid,
    pub reading: GarageDoorReading,
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

pub struct GarageDoorHandler {
    shared_actor_state: AppState,
}

impl GarageDoorHandler {
    pub const NAME: &str = "garage_door";

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
                    Ok(outcome) => tracing::info!("garage door {address} {command}: {outcome}"),
                    Err(e) => tracing::warn!("garage door {address} {command} failed: {e}"),
                }

                if reply.send(outcome).is_err() {
                    tracing::debug!("garage door {address} {command} caller went away");
                }

                Ok(())
            }
        }
    }

    async fn command(
        &self,
        decoders: &Decoders,
        address: &str,
        command: GarageDoorCommand,
    ) -> Result<CommandOutcome, String> {
        let devices = &self.shared_actor_state.devices;

        let Some(settings) = devices.garage_door(address) else {
            return Err(format!("{address} is not a garage door"));
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
            .garage_door()
            .latest(&settings.id)
            .await
            .map_err(|e| format!("failed to load the current state: {e}"))?
            .map(|row| row.state.to_string());

        let input = json!({
            "command": command.to_string(),
            "current": current,
        });

        let payload = profile
            .encode::<_, serde_json::Value>(decoder, DeviceRoleName::GarageDoor, &input)
            .map_err(|e| format!("failed to encode {command}: {e}"))?;

        let Some(payload) = payload else {
            return Ok(CommandOutcome::Unchanged);
        };

        let targets = CommandTargets::new(devices, &self.shared_actor_state.handles);

        device_command::send(
            &targets,
            address,
            DeviceRoleName::GarageDoor,
            Outbound::Json(payload),
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(CommandOutcome::Sent)
    }

    async fn record(
        &self,
        event_id: Uuid,
        reading: GarageDoorReading,
    ) -> Result<(), anyhow::Error> {
        let GarageDoorReading {
            address,
            state,
            contact,
        } = reading;

        let Some(settings) = self.shared_actor_state.devices.garage_door(&address) else {
            tracing::warn!("ignoring garage door reading for undeclared device {address}");
            return Ok(());
        };

        let previous = self
            .shared_actor_state
            .repos
            .garage_door()
            .record(event_id, &settings.id, state, contact)
            .await?;

        if previous == Some(state) {
            tracing::trace!("garage door {} is still {state}", settings.id);
            return Ok(());
        }

        tracing::info!("garage door {} is {state}", settings.id);

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::GarageDoor {
                event_id,
                device_id: settings.id.clone(),
                name: settings.name.clone(),
                state,
            });

        let left_closed = previous.is_none_or(|previous| previous == GarageDoorState::Closed)
            && state != GarageDoorState::Closed;

        if left_closed {
            self.arm(settings);
        }

        Ok(())
    }

    fn arm(&self, settings: &GarageDoorSettings) {
        let timeout = match settings.armed {
            ArmedDoorStates::Armed { timeout } => timeout,
            ArmedDoorStates::Unarmed => {
                tracing::debug!("garage door {} is unarmed", settings.id);
                return;
            }
        };

        let Ok(delay) = timeout.to_std() else {
            tracing::error!("garage door {} has a negative arming timeout", settings.id);
            return;
        };

        let repos = self.shared_actor_state.repos.clone();
        let settings = settings.clone();
        let opened_at = Utc::now();

        tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let repo = repos.garage_door();

            let closed = match repo.closed_since(&settings.id, opened_at).await {
                Ok(closed) => closed,
                Err(e) => {
                    tracing::error!("failed to check garage door {}: {e}", settings.id);
                    return;
                }
            };

            let open = match repo.latest(&settings.id).await {
                Ok(latest) => latest.is_some_and(|row| row.state != GarageDoorState::Closed),
                Err(e) => {
                    tracing::error!("failed to load garage door {}: {e}", settings.id);
                    return;
                }
            };

            if closed || !open {
                tracing::debug!("garage door {} closed within its timeout", settings.id);
                return;
            }

            tracing::info!("garage door {} has been left open", settings.id);

            notify(
                &settings.notify,
                Notification::new(
                    NotificationSource::GarageDoorLeftOpen,
                    format!("{} has been left open.", settings.name),
                    NotifyCategory::Door,
                    format!("garage_door:{}", settings.id),
                ),
            );
        });
    }
}

impl DeviceHandler for GarageDoorHandler {
    const NAME: &'static str = GarageDoorHandler::NAME;

    type Message = Message;
    type State = Decoders;

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    const ROLE: DeviceRoleName = DeviceRoleName::GarageDoor;

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
