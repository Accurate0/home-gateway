pub mod lua;

use std::collections::HashMap;

use crate::actors::devices::handler::DeviceHandler;
use crate::{event_bus::EventBusMessage, state::AppState};
use ractor::RpcReplyPort;
use uuid::Uuid;

pub struct Entity {
    pub address: String,
    pub sensor: Option<String>,
    pub presence: bool,
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
    pub traceparent: crate::tracing_context::TraceParent,
}

pub enum Message {
    NewEvent(NewEvent),
    /// Last known presence for a sensor (keyed by ieee address or esphome node),
    /// or `None` if the sensor hasn't reported since startup.
    QueryLatest {
        sensor: String,
        reply: RpcReplyPort<Option<bool>>,
    },
}

impl crate::tracing_context::TracedMessage for Message {
    fn traceparent(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => event.traceparent.as_deref(),
            Message::QueryLatest { .. } => None,
        }
    }

    fn subject(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => Some(&event.entity.address),
            Message::QueryLatest { sensor, .. } => Some(sensor),
        }
    }
}

#[derive(Default)]
pub struct PresenceSensorState {
    pub last_presence: HashMap<String, bool>,
    pub sensors: HashMap<String, HashMap<String, bool>>,
}

pub struct PresenceSensorHandler {
    shared_actor_state: AppState,
}

impl PresenceSensorHandler {
    pub const NAME: &str = "presence-sensor";

    fn process_presence(
        &self,
        event_id: Uuid,
        key: String,
        presence: bool,
        state: &mut PresenceSensorState,
    ) -> Result<(), anyhow::Error> {
        let mut was_state_changed = true;
        state
            .last_presence
            .entry(key.clone())
            .and_modify(|prev| {
                if *prev != presence {
                    *prev = presence;
                } else {
                    was_state_changed = false;
                }
            })
            .or_insert(presence);

        // edge-detected here so triggers receive discrete transitions, not every
        // sensor ping
        if was_state_changed {
            self.shared_actor_state
                .event_bus
                .publish(EventBusMessage::Presence {
                    event_id,
                    sensor: key,
                    present: presence,
                });
        }

        Ok(())
    }

    async fn handle(
        &self,
        message: Message,
        state: &mut PresenceSensorState,
    ) -> Result<(), anyhow::Error> {
        match message {
            Message::QueryLatest { sensor, reply } => {
                reply.send(state.last_presence.get(&sensor).copied())?;
            }
            Message::NewEvent(event) => {
                let Entity {
                    address,
                    sensor,
                    presence,
                } = event.entity;

                let present = match sensor {
                    Some(sensor) => {
                        let sensors = state.sensors.entry(address.clone()).or_default();
                        sensors.insert(sensor, presence);

                        sensors.values().any(|&on| on)
                    }
                    None => presence,
                };

                self.process_presence(event.event_id, address, present, state)?
            }
        }

        Ok(())
    }
}

impl DeviceHandler for PresenceSensorHandler {
    const NAME: &'static str = PresenceSensorHandler::NAME;

    type Message = Message;
    type State = PresenceSensorState;

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    fn workers(workers: &crate::settings::ActorWorkerSettings) -> usize {
        workers.presence_sensor
    }

    async fn handle(&self, message: Self::Message, state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message, state).await
    }
}
