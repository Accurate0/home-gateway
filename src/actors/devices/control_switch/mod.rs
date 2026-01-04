use crate::{
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage},
    settings::{IEEEAddress, SwitchSettings, workflow::WorkflowSettings},
    types::SharedActorState,
    zigbee2mqtt::{Aqara_WXKG11LM, IKEA_E2001},
};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, JobOptions, Worker, WorkerBuilder, WorkerId},
};
use std::collections::HashMap;
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
    #[allow(unused)]
    shared_actor_state: SharedActorState,
    switch_settings: HashMap<IEEEAddress, SwitchSettings>,
}

impl ControlSwitchHandler {
    pub const NAME: &str = "control-switch";

    fn execute_workflow(
        event_id: Uuid,
        workflow_settings: &WorkflowSettings,
    ) -> Result<(), anyhow::Error> {
        let Some(actor) = ractor::registry::where_is(WorkflowWorker::NAME.to_string()) else {
            tracing::warn!("actor not found for workflow");
            return Ok(());
        };

        let message = FactoryMessage::Dispatch(Job {
            key: (),
            msg: WorkflowWorkerMessage::Execute {
                event_id,
                workflow: workflow_settings.to_owned(),
            },
            options: JobOptions::default(),
            accepted: None,
        });

        actor.send_message(message)?;

        Ok(())
    }

    async fn handle(&self, message: ControlSwitchMessage) -> Result<(), anyhow::Error> {
        match message {
            ControlSwitchMessage::NewEvent(event) => match &event.entity {
                Entity::IKEASwitch(ikea_e20001) => {
                    let Some(action_settings) = self
                        .switch_settings
                        .get(&ikea_e20001.device.ieee_addr)
                        .and_then(|s| s.actions.get(&ikea_e20001.action))
                    else {
                        tracing::warn!("no valid action found for: {:?}", &event);
                        return Ok(());
                    };

                    Self::execute_workflow(event.event_id, &action_settings.workflow)?;
                }
                Entity::AqaraSingleButton(aqara_wxkg11_lm) => {
                    // ignore empty action
                    if aqara_wxkg11_lm.action == "" {
                        return Ok(());
                    }

                    let Some(action_settings) = self
                        .switch_settings
                        .get(&aqara_wxkg11_lm.device.ieee_addr)
                        .and_then(|s| s.actions.get(&aqara_wxkg11_lm.action))
                    else {
                        tracing::warn!("no valid action found for: {:?}", &event);
                        return Ok(());
                    };

                    Self::execute_workflow(event.event_id, &action_settings.workflow)?;
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

    #[tracing::instrument(name = "control-switch", skip(self, _wid, _factory, msg, _state))]
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
    pub switch_settings: HashMap<IEEEAddress, SwitchSettings>,
}
impl WorkerBuilder<ControlSwitchHandler, ()> for ControlSwitchHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (ControlSwitchHandler, ()) {
        (
            ControlSwitchHandler {
                shared_actor_state: self.shared_actor_state.clone(),
                switch_settings: self.switch_settings.clone(),
            },
            (),
        )
    }
}
