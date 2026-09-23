use crate::actors::devices::handler::DeviceHandler;
use crate::repo::plant::PlantReading;
use crate::{event_bus::EventBusMessage, state::AppState};
use uuid::Uuid;

pub struct Entity {
    pub address: String,
    pub soil_moisture: f64,
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
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
            Message::NewEvent(event) => Some(&event.entity.address),
        }
    }
}

pub struct PlantSensorHandler {
    shared_actor_state: AppState,
}

impl PlantSensorHandler {
    pub const NAME: &str = "plant-sensor";

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        let Message::NewEvent(event) = message;
        let Entity {
            address,
            soil_moisture,
        } = event.entity;

        if let Some(settings) = self.shared_actor_state.devices.plant(&address) {
            self.shared_actor_state
                .repos
                .plant()
                .record(&PlantReading {
                    event_id: event.event_id,
                    id: Some(settings.id.clone()),
                    friendly_name: settings.name.clone(),
                    address: address.clone(),
                    soil_moisture,
                })
                .await?;
        }

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Plant {
                event_id: event.event_id,
                sensor: address,
                soil_moisture,
            });

        Ok(())
    }
}

impl DeviceHandler for PlantSensorHandler {
    const NAME: &'static str = PlantSensorHandler::NAME;

    type Message = Message;
    type State = ();

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    const ROLE: crate::decoding::DeviceRoleName = crate::decoding::DeviceRoleName::Plant;

    fn init_state(&self) -> anyhow::Result<Self::State> {
        Ok(())
    }

    async fn handle(&self, message: Self::Message, _state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message).await
    }
}
