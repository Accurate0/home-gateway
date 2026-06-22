use crate::{
    event_bus::EventBusMessage,
    types::SharedActorState,
    zigbee2mqtt::{Aqara_WXKG11LM, IKEA_E2001},
};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

#[derive(Debug)]
pub enum Entity {
    IKEASwitch(IKEA_E2001::IKEAE2001),
    AqaraSingleButton(Aqara_WXKG11LM::AqaraWXKG11LM),
}

#[derive(Debug)]
pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
}

pub enum ControlSwitchMessage {
    NewEvent(NewEvent),
}

pub struct ControlSwitchHandler {
    shared_actor_state: SharedActorState,
}

impl ControlSwitchHandler {
    pub const NAME: &str = "control-switch";

    async fn handle(&self, message: ControlSwitchMessage) -> Result<(), anyhow::Error> {
        match message {
            ControlSwitchMessage::NewEvent(event) => match &event.entity {
                Entity::IKEASwitch(ikea_e20001) => {
                    self.shared_actor_state
                        .event_bus
                        .publish(EventBusMessage::SwitchAction {
                            event_id: event.event_id,
                            ieee_addr: ikea_e20001.device.ieee_addr.clone(),
                            action: ikea_e20001.action.clone(),
                        });
                }
                Entity::AqaraSingleButton(aqara_wxkg11_lm) => {
                    // ignore empty action
                    if aqara_wxkg11_lm.action.is_empty() {
                        return Ok(());
                    }

                    self.shared_actor_state
                        .event_bus
                        .publish(EventBusMessage::SwitchAction {
                            event_id: event.event_id,
                            ieee_addr: aqara_wxkg11_lm.device.ieee_addr.clone(),
                            action: aqara_wxkg11_lm.action.clone(),
                        });
                }
            },
        }

        Ok(())
    }
}

impl Worker for ControlSwitchHandler {
    type Key = ();
    type Message = ControlSwitchMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), ControlSwitchMessage>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), ControlSwitchMessage>>,
        Job { msg, .. }: Job<(), ControlSwitchMessage>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg).await {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct ControlSwitchHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}
impl WorkerBuilder<ControlSwitchHandler, ()> for ControlSwitchHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (ControlSwitchHandler, ()) {
        (
            ControlSwitchHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
