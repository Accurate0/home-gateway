pub mod command;
pub mod lua;
mod roborock_reading;

pub use roborock_reading::RoborockReading;

use crate::actors::devices::handler::DeviceHandler;
use crate::actors::system::battery::BatteryActor;
use crate::state::AppState;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leaf {
    State,
    Attributes,
}

pub struct ValetudoEvent {
    pub event_id: Uuid,
    pub device_id: String,
    pub leaf: Leaf,
    pub payload: bytes::Bytes,
    pub traceparent: crate::tracing_context::TraceParent,
}

pub struct RoborockUpdate {
    pub event_id: Uuid,
    pub reading: RoborockReading,
    pub traceparent: crate::tracing_context::TraceParent,
}

pub enum Message {
    Valetudo(ValetudoEvent),
    Roborock(RoborockUpdate),
}

impl crate::tracing_context::TracedMessage for Message {
    fn traceparent(&self) -> Option<&str> {
        match self {
            Message::Valetudo(event) => event.traceparent.as_deref(),
            Message::Roborock(update) => update.traceparent.as_deref(),
        }
    }

    fn subject(&self) -> Option<&str> {
        match self {
            Message::Valetudo(event) => Some(&event.device_id),
            Message::Roborock(update) => Some(&update.reading.device_id),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ValetudoState {
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    battery_level: Option<i32>,
    #[serde(default)]
    fan_speed: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ValetudoAttributes {
    #[serde(
        default,
        rename = "currentCleanArea",
        deserialize_with = "crate::serde_lenient::opt_f64"
    )]
    current_clean_area: Option<f64>,
    #[serde(default, rename = "cleanCount")]
    clean_count: Option<i32>,
}

pub struct RobotVacuumHandler {
    shared_actor_state: AppState,
}

impl RobotVacuumHandler {
    pub const NAME: &str = "robot_vacuum";

    fn device_name(&self, device_id: &str) -> String {
        let devices = &self.shared_actor_state.devices;
        let address = devices.address_or_self(device_id);

        devices
            .valetudo(address)
            .map(|v| v.name.clone())
            .or_else(|| devices.roborock(address).map(|r| r.name.clone()))
            .unwrap_or_else(|| device_id.to_owned())
    }

    async fn handle_valetudo_state(
        &self,
        event_id: Uuid,
        device_id: &str,
        payload: &[u8],
    ) -> Result<(), anyhow::Error> {
        let parsed = serde_json::from_slice::<ValetudoState>(payload)?;

        self.shared_actor_state
            .repos
            .robot_vacuum()
            .record_valetudo_state(
                event_id,
                device_id,
                parsed.state.as_deref(),
                parsed.battery_level,
                parsed.fan_speed.as_deref(),
            )
            .await?;

        if let Some(level) = parsed.battery_level {
            self.report_battery(device_id, level);
        }

        Ok(())
    }

    async fn handle_valetudo_attributes(
        &self,
        device_id: &str,
        payload: &[u8],
    ) -> Result<(), anyhow::Error> {
        let parsed = serde_json::from_slice::<ValetudoAttributes>(payload)?;
        let attributes = serde_json::from_slice::<serde_json::Value>(payload)?;

        self.shared_actor_state
            .repos
            .robot_vacuum()
            .upsert_valetudo_attributes(
                device_id,
                parsed.current_clean_area,
                parsed.clean_count,
                &attributes,
            )
            .await?;

        Ok(())
    }

    async fn handle_roborock(&self, update: RoborockUpdate) -> Result<(), anyhow::Error> {
        let RoborockUpdate {
            event_id,
            reading:
                RoborockReading {
                    device_id,
                    status,
                    room,
                    battery,
                },
            ..
        } = update;

        let repo = self.shared_actor_state.repos.robot_vacuum();

        match status {
            Some(status) => {
                repo.record_roborock_status(event_id, &device_id, &status)
                    .await?
            }
            None => tracing::trace!("no roborock status in this update for {device_id}"),
        }

        match battery {
            Some(level) => {
                let level = level as i32;

                repo.record_roborock_battery(event_id, &device_id, Some(level))
                    .await?;
                self.report_battery(&device_id, level);
            }
            None => tracing::trace!("no roborock battery in this update for {device_id}"),
        }

        match room {
            Some(room) => repo.upsert_roborock_room(&device_id, &room).await?,
            None => tracing::trace!("no roborock room in this update for {device_id}"),
        }

        Ok(())
    }

    fn report_battery(&self, device_id: &str, level: i32) {
        let name = self.device_name(device_id);

        BatteryActor::report(
            device_id.to_owned(),
            name,
            Self::NAME.to_owned(),
            None,
            Some(level as f64),
            None,
        );
    }

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::Valetudo(event) => match event.leaf {
                Leaf::State => {
                    self.handle_valetudo_state(event.event_id, &event.device_id, &event.payload)
                        .await?
                }
                Leaf::Attributes => {
                    self.handle_valetudo_attributes(&event.device_id, &event.payload)
                        .await?
                }
            },
            Message::Roborock(update) => self.handle_roborock(update).await?,
        }

        Ok(())
    }
}

impl DeviceHandler for RobotVacuumHandler {
    const NAME: &'static str = RobotVacuumHandler::NAME;

    type Message = Message;
    type State = ();

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    fn workers(workers: &crate::settings::ActorWorkerSettings) -> usize {
        workers.robot_vacuum
    }

    async fn handle(&self, message: Self::Message, _state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message).await
    }
}
