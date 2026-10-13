mod air_purifier_command_error;
mod air_purifier_reading;
pub mod command;
mod command_request;
pub mod lua;
mod reapply;

pub use air_purifier_command_error::AirPurifierCommandError;
pub use air_purifier_reading::AirPurifierReading;
pub use command_request::CommandRequest;
pub use reapply::Reapply;

use serde_json::{Value, json};
use uuid::Uuid;

use crate::actors::devices::handler::DeviceHandler;
use crate::decoding::{Decoders, DeviceRoleName};
use crate::device_command::{self, CommandOrigin, CommandOutcome, CommandTargets, Outbound};
use crate::device_lock::{self, DeviceLocks, LockedDevice};
use crate::event_bus::EventBusMessage;
use crate::repo::air_purifier::{AirPurifierRecord, AirPurifierState};
use crate::repo::intent::{DeviceKind, DeviceReport, IntentAttributes, IntentStatus};
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
    Reapply(Reapply),
}

impl crate::telemetry::context::TracedMessage for Message {
    fn traceparent(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => event.traceparent.as_deref(),
            Message::Command(request) => request.traceparent.as_deref(),
            Message::Reapply(reapply) => reapply.traceparent.as_deref(),
        }
    }

    fn subject(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => Some(&event.reading.address),
            Message::Command(request) => Some(&request.address),
            Message::Reapply(reapply) => Some(&reapply.address),
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
                origin,
                reply,
                ..
            }) => {
                let outcome = self.command(decoders, &address, command, origin).await;

                match &outcome {
                    Ok(outcome) => {
                        tracing::info!("air purifier {address} {command} ({origin}): {outcome}")
                    }
                    Err(e) => {
                        tracing::warn!("air purifier {address} {command} ({origin}) failed: {e}")
                    }
                }

                if reply.send(outcome).is_err() {
                    tracing::debug!("air purifier {address} {command} caller went away");
                }

                Ok(())
            }
            Message::Reapply(Reapply {
                address,
                attributes,
                ..
            }) => {
                for command in AirPurifierCommand::reapply(&attributes) {
                    match self.resend(decoders, &address, command).await {
                        Ok(outcome) => {
                            tracing::info!("air purifier {address} re-drove {command}: {outcome}")
                        }
                        Err(e) => {
                            tracing::warn!(
                                "air purifier {address} failed to re-drive {command}: {e}"
                            )
                        }
                    }
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
        origin: CommandOrigin,
    ) -> Result<CommandOutcome, String> {
        let state = &self.shared_actor_state;

        let Some(settings) = state.devices.air_purifier(address) else {
            return Err(format!("{address} is not an air purifier"));
        };

        let lock = DeviceLocks::new(&state.repos, &state.event_bus)
            .guard(origin, &settings.id)
            .await
            .map_err(|e| format!("failed to read the lock: {e}"))?;

        if let Some(lock) = lock {
            tracing::info!(
                "air purifier {} is locked ({}), skipping {command}",
                settings.id,
                device_lock::describe(&lock)
            );

            return Ok(CommandOutcome::Locked);
        }

        let Some(payload) = self.encode(decoders, address, command).await? else {
            return Ok(CommandOutcome::Unchanged);
        };

        let intent = self.record_intent(address, command).await;

        if let Err(e) = self.dispatch(address, payload).await {
            self.fail_intent(address, intent).await;

            return Err(e);
        }

        Ok(CommandOutcome::Sent)
    }

    async fn resend(
        &self,
        decoders: &Decoders,
        address: &str,
        command: AirPurifierCommand,
    ) -> Result<CommandOutcome, String> {
        let Some(payload) = self.encode(decoders, address, command).await? else {
            return Ok(CommandOutcome::Unchanged);
        };

        self.dispatch(address, payload).await?;

        Ok(CommandOutcome::Sent)
    }

    async fn record_intent(&self, address: &str, command: AirPurifierCommand) -> Option<i64> {
        let repo = self.shared_actor_state.repos.intent();
        let wanted = command.attributes();

        let pending = match repo.pending_for(DeviceKind::AirPurifier, address).await {
            Ok(pending) => pending,
            Err(e) => {
                tracing::warn!("failed to read pending intents for {address}: {e}");
                Vec::new()
            }
        };

        let merged = pending
            .last()
            .and_then(|intent| intent.attributes.as_air_purifier())
            .map_or(wanted, |pending| wanted.merged_over(pending));

        match repo
            .replace_pending(
                address,
                &IntentAttributes::AirPurifier(merged),
                false,
                Uuid::new_v4(),
            )
            .await
        {
            Ok(id) => Some(id),
            Err(e) => {
                tracing::warn!("failed to record intent for {address}: {e}");
                None
            }
        }
    }

    async fn fail_intent(&self, address: &str, intent: Option<i64>) {
        let Some(id) = intent else {
            return;
        };

        if let Err(e) = self
            .shared_actor_state
            .repos
            .intent()
            .settle(&[id], IntentStatus::Failed)
            .await
        {
            tracing::warn!("failed to mark intent {id} for {address} as failed: {e}");
        }
    }

    async fn encode(
        &self,
        decoders: &Decoders,
        address: &str,
        command: AirPurifierCommand,
    ) -> Result<Option<Value>, String> {
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

        profile
            .encode::<_, Value>(decoder, DeviceRoleName::AirPurifier, &input)
            .map_err(|e| format!("failed to encode {command}: {e}"))
    }

    async fn dispatch(&self, address: &str, payload: Value) -> Result<(), String> {
        let state = &self.shared_actor_state;
        let targets = CommandTargets::new(&state.devices, &state.handles);

        device_command::send(
            &targets,
            address,
            DeviceRoleName::AirPurifier,
            Outbound::Json(payload),
        )
        .await
        .map_err(|e| e.to_string())
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

        let current = AirPurifierState {
            on,
            mode,
            speed,
            display,
        };
        let before = previous.as_ref().map(AirPurifierState::from);

        DeviceLocks::new(
            &self.shared_actor_state.repos,
            &self.shared_actor_state.event_bus,
        )
        .observe(
            &LockedDevice {
                device_id: &settings.id,
                address: &address,
                settings: &settings.lock,
            },
            before.as_ref().map(DeviceReport::AirPurifier).as_ref(),
            &DeviceReport::AirPurifier(&current),
        )
        .await;

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
