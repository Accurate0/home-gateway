use std::collections::BTreeMap;

use crate::actors::devices::handler::DeviceHandler;
use crate::{
    event_bus::{EventBusMessage, SensorReading},
    state::AppState,
};
use uuid::Uuid;

pub struct Entity {
    pub address: String,
    pub readings: BTreeMap<String, f64>,
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

    fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        let Message::NewEvent(event) = message;
        let Entity { address, readings } = event.entity;

        let readings = readings
            .into_iter()
            .map(|(metric, value)| SensorReading::new(metric.into(), value))
            .collect();

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Environment {
                event_id: event.event_id,
                sensor: address,
                readings,
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

    fn workers(workers: &crate::settings::ActorWorkerSettings) -> usize {
        workers.plant_sensor
    }

    async fn handle(&self, message: Self::Message, _state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message)
    }
}
