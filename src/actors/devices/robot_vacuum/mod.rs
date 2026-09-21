pub mod command;
pub mod lua;
mod robot_vacuum_reading;

pub use robot_vacuum_reading::RobotVacuumReading;

use crate::actors::devices::handler::DeviceHandler;
use crate::actors::system::battery::BatteryActor;
use crate::integrations::mqtt::MqttProtocol;
use crate::state::AppState;
use uuid::Uuid;

pub struct NewEvent {
    pub event_id: Uuid,
    pub reading: RobotVacuumReading,
    pub traceparent: crate::tracing_context::TraceParent,
}

pub enum Message {
    NewEvent(NewEvent),
}

impl crate::tracing_context::TracedMessage for Message {
    fn traceparent(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => event.traceparent.as_deref(),
        }
    }

    fn subject(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => Some(&event.reading.device_id),
        }
    }
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
            .robot_vacuum(address)
            .map_or_else(|| device_id.to_owned(), |settings| settings.name.clone())
    }

    async fn record_home_assistant(
        &self,
        event_id: Uuid,
        reading: RobotVacuumReading,
    ) -> Result<(), anyhow::Error> {
        let RobotVacuumReading {
            device_id,
            status,
            room,
            battery,
            ..
        } = reading;

        let repo = self.shared_actor_state.repos.robot_vacuum();

        match status {
            Some(status) => {
                repo.record_roborock_status(event_id, &device_id, &status)
                    .await?
            }
            None => tracing::trace!("no robot vacuum status in this update for {device_id}"),
        }

        match battery {
            Some(level) => {
                let level = level as i32;

                repo.record_roborock_battery(event_id, &device_id, Some(level))
                    .await?;
                self.report_battery(&device_id, level);
            }
            None => tracing::trace!("no robot vacuum battery in this update for {device_id}"),
        }

        match room {
            Some(room) => repo.upsert_roborock_room(&device_id, &room).await?,
            None => tracing::trace!("no robot vacuum room in this update for {device_id}"),
        }

        Ok(())
    }

    async fn record_valetudo(
        &self,
        event_id: Uuid,
        reading: RobotVacuumReading,
    ) -> Result<(), anyhow::Error> {
        let RobotVacuumReading {
            device_id,
            status,
            battery,
            fan_speed,
            clean_area,
            clean_count,
            attributes,
            ..
        } = reading;

        let repo = self.shared_actor_state.repos.robot_vacuum();

        match attributes {
            Some(attributes) => {
                repo.upsert_valetudo_attributes(&device_id, clean_area, clean_count, &attributes)
                    .await?
            }
            None => {
                let level = battery.map(|level| level as i32);

                repo.record_valetudo_state(
                    event_id,
                    &device_id,
                    status.as_deref(),
                    level,
                    fan_speed.as_deref(),
                )
                .await?;

                match level {
                    Some(level) => self.report_battery(&device_id, level),
                    None => tracing::trace!("no valetudo battery in this update for {device_id}"),
                }
            }
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
        let Message::NewEvent(NewEvent {
            event_id, reading, ..
        }) = message;

        match reading.protocol {
            None => self.record_home_assistant(event_id, reading).await?,
            Some(MqttProtocol::Valetudo) => self.record_valetudo(event_id, reading).await?,
            Some(protocol @ (MqttProtocol::Zigbee | MqttProtocol::Esphome)) => tracing::warn!(
                "ignoring robot vacuum reading from a {protocol} device {}",
                reading.device_id
            ),
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
