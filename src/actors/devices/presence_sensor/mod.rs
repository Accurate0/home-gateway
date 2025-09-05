use std::collections::HashMap;

use crate::{
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage},
    settings::{IEEEAddress, PresenceActionId, PresenceSettings, workflow::WorkflowSettings},
    types::SharedActorState,
    zigbee2mqtt::Aqara_FP1E,
};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, JobOptions, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub enum Entity {
    AqaraFP1E(Aqara_FP1E::AqaraFP1E),
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
}

pub enum Message {
    NewEvent(NewEvent),
}

pub struct PresenceSensorHandler {
    _shared_actor_state: SharedActorState,
    presence_settings: HashMap<IEEEAddress, PresenceSettings>,
}

impl PresenceSensorHandler {
    pub const NAME: &str = "presence-sensor";

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

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::NewEvent(event) => match event.entity {
                Entity::AqaraFP1E(aqara_fp1_e) => {
                    let Some(presence_settings) =
                        self.presence_settings.get(&aqara_fp1_e.device.ieee_addr)
                    else {
                        tracing::warn!(
                            "no valid setting found for: {}",
                            aqara_fp1_e.device.ieee_addr
                        );
                        return Ok(());
                    };

                    // TODO: ignore recent events where the presence has not changed
                    // the presence sensor is quite loud
                    let action_settings = if aqara_fp1_e.presence {
                        presence_settings
                            .actions
                            .get(&PresenceActionId::PresenceDetected)
                    } else {
                        presence_settings
                            .actions
                            .get(&PresenceActionId::NoPresenceDetected)
                    };

                    let Some(action_settings) = action_settings else {
                        tracing::warn!("no set action for event");
                        return Ok(());
                    };

                    Self::execute_workflow(event.event_id, &action_settings.workflow)?;
                }
            },
        }

        Ok(())
    }
}

impl Worker for PresenceSensorHandler {
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

    #[tracing::instrument(name = "presence-sensor", skip(self, _wid, _factory, msg, _state))]
    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        Job { msg, .. }: Job<(), Message>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg).await {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct PresenceSensorHandlerBuilder {
    pub shared_actor_state: SharedActorState,
    pub presence_settings: HashMap<IEEEAddress, PresenceSettings>,
}

impl WorkerBuilder<PresenceSensorHandler, ()> for PresenceSensorHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (PresenceSensorHandler, ()) {
        (
            PresenceSensorHandler {
                _shared_actor_state: self.shared_actor_state.clone(),
                presence_settings: self.presence_settings.clone(),
            },
            (),
        )
    }
}
