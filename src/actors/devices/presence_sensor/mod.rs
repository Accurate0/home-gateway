use std::collections::HashMap;

use crate::{event_bus::EventBusMessage, types::SharedActorState, zigbee2mqtt::Aqara_FP1E};
use ractor::{
    ActorProcessingErr, ActorRef, RpcReplyPort,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub enum Entity {
    AqaraFP1E(Box<Aqara_FP1E::AqaraFP1E>),
    Esphome { node: String, motion: bool },
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

pub struct PresenceSensorState {
    pub last_presence: HashMap<String, bool>,
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
                Entity::AqaraFP1E(aqara_fp1_e) => self.process_presence(
                    event.event_id,
                    aqara_fp1_e.device.ieee_addr,
                    aqara_fp1_e.presence,
                    state,
                )?,
                Entity::Esphome { node, motion } => {
                    self.process_presence(event.event_id, node, motion, state)?
                }
            },
        }

        Ok(())
    }
}

impl Worker for PresenceSensorHandler {
    type Key = ();
    type Message = Message;
    type State = PresenceSensorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(PresenceSensorState {
            last_presence: Default::default(),
        })
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        Job { msg, .. }: Job<(), Message>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg, state).await {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct PresenceSensorHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}

impl WorkerBuilder<PresenceSensorHandler, ()> for PresenceSensorHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (PresenceSensorHandler, ()) {
        (
            PresenceSensorHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
