use crate::{
    event_bus::{EventBusMessage, SensorReading},
    types::SharedActorState,
};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub struct NewEvent {
    pub event_id: Uuid,
    /// esphome node name, used as the plant sensor's settings key.
    pub node: String,
    /// esphome sensor object_id that produced this reading (e.g. `soil_moisture`).
    pub object_id: String,
    pub value: f64,
}

pub enum Message {
    NewEvent(NewEvent),
}

pub struct PlantSensorHandler {
    shared_actor_state: SharedActorState,
}

impl PlantSensorHandler {
    pub const NAME: &str = "plant-sensor";

    /// Publish each reading onto the bus as an `Environment` event keyed by node
    /// and object_id. Thresholds and rising-edge handling live in the dispatcher
    /// (driven by `triggers:`).
    fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        let Message::NewEvent(event) = message;

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Environment {
                event_id: event.event_id,
                sensor: event.node,
                readings: vec![SensorReading::new(event.object_id.into(), event.value)],
            });

        Ok(())
    }
}

impl Worker for PlantSensorHandler {
    type Key = ();
    type Message = Message;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        Job { msg, .. }: Job<(), Message>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg) {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct PlantSensorHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}

impl WorkerBuilder<PlantSensorHandler, ()> for PlantSensorHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (PlantSensorHandler, ()) {
        (
            PlantSensorHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
