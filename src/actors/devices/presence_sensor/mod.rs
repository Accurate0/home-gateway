use std::collections::HashMap;

use crate::actors::devices::handler::DeviceHandler;
use crate::{event_bus::EventBusMessage, state::SharedActorState};
use ractor::RpcReplyPort;
use uuid::Uuid;

pub enum Entity {
    Zigbee {
        address: String,
        presence: bool,
    },
    Esphome {
        node: String,
        object_id: String,
        motion: bool,
    },
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
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

#[derive(Default)]
pub struct PresenceSensorState {
    pub last_presence: HashMap<String, bool>,
    pub esphome_entities: HashMap<String, HashMap<String, bool>>,
}

pub struct PresenceSensorHandler {
    shared_actor_state: SharedActorState,
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
            Message::NewEvent(event) => match event.entity {
                Entity::Zigbee { address, presence } => {
                    self.process_presence(event.event_id, address, presence, state)?
                }
                Entity::Esphome {
                    node,
                    object_id,
                    motion,
                } => {
                    let entities = state.esphome_entities.entry(node.clone()).or_default();
                    entities.insert(object_id, motion);
                    let present = entities.values().any(|&on| on);
                    self.process_presence(event.event_id, node, present, state)?
                }
            },
        }

        Ok(())
    }
}

impl DeviceHandler for PresenceSensorHandler {
    const NAME: &'static str = PresenceSensorHandler::NAME;

    type Message = Message;
    type State = PresenceSensorState;

    fn new(shared_actor_state: SharedActorState) -> Self {
        Self { shared_actor_state }
    }

    async fn handle(&self, message: Self::Message, state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message, state).await
    }
}
