use crate::actors::devices::handler::DeviceHandler;
use crate::{
    event_bus::{EventBusMessage, SensorReading},
    state::SharedActorState,
};
use uuid::Uuid;

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

impl DeviceHandler for PlantSensorHandler {
    const NAME: &'static str = PlantSensorHandler::NAME;
    const WORKERS: usize = 1;

    type Message = Message;
    type State = ();

    fn new(shared_actor_state: SharedActorState) -> Self {
        Self { shared_actor_state }
    }

    async fn handle(&self, message: Self::Message, _state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message)
    }
}
