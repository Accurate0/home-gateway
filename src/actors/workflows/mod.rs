use crate::{
    actors::light::{LightHandler, LightHandlerMessage},
    settings::workflow::{WorkflowEntityLightTypeState, WorkflowEntityType, WorkflowSettings},
    timer::timed_async,
};
use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, JobOptions, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub enum WorkflowWorkerMessage {
    Execute {
        event_id: Uuid,
        workflow: WorkflowSettings,
    },
}

pub struct WorkflowWorker {}

impl WorkflowWorker {
    pub const NAME: &str = "workflow";

    fn handle_light_operation(
        ieee_addr: String,
        state: WorkflowEntityLightTypeState,
    ) -> Result<(), anyhow::Error> {
        let Some(actor) = ractor::registry::where_is(LightHandler::NAME.to_string()) else {
            tracing::warn!("could not find light actor");
            return Ok(());
        };

        let light_actor_message = match state {
            WorkflowEntityLightTypeState::On => LightHandlerMessage::TurnOn { ieee_addr },
            WorkflowEntityLightTypeState::Off => LightHandlerMessage::TurnOff { ieee_addr },
            WorkflowEntityLightTypeState::Toggle => LightHandlerMessage::Toggle { ieee_addr },
            WorkflowEntityLightTypeState::IncreaseBrightness { value } => {
                LightHandlerMessage::BrightnessMove {
                    ieee_addr,
                    value: value.try_into()?,
                }
            }
            WorkflowEntityLightTypeState::DecreaseBrightness { value } => {
                LightHandlerMessage::BrightnessMove {
                    ieee_addr,
                    value: -value.try_into()?,
                }
            }
            WorkflowEntityLightTypeState::StopBrightness => LightHandlerMessage::BrightnessMove {
                ieee_addr,
                value: 0,
            },
            WorkflowEntityLightTypeState::IncreaseColourTemperature { value } => {
                LightHandlerMessage::ColourTemperatureMove {
                    ieee_addr,
                    value: value.try_into()?,
                }
            }
            WorkflowEntityLightTypeState::DecreaseColourTemperature { value } => {
                LightHandlerMessage::ColourTemperatureMove {
                    ieee_addr,
                    value: -value.try_into()?,
                }
            }
            WorkflowEntityLightTypeState::StopColourTemperature => {
                LightHandlerMessage::ColourTemperatureMove {
                    ieee_addr,
                    value: 0,
                }
            }
        };

        let message = FactoryMessage::Dispatch(Job {
            key: (),
            msg: light_actor_message,
            options: JobOptions::default(),
            accepted: None,
        });
        actor.send_message(message)?;

        Ok(())
    }

    pub async fn execute_workflow(
        event_id: Uuid,
        workflow: WorkflowSettings,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("executing workflow for: {event_id}");
        for step in workflow.run {
            match step {
                WorkflowEntityType::Light { ieee_addr, state } => {
                    tokio::runtime::Handle::current()
                        .spawn_blocking(|| {
                            Self::handle_light_operation(ieee_addr, state)?;
                            Ok::<(), anyhow::Error>(())
                        })
                        .await??;
                }
            }
        }

        Ok(())
    }
}

impl Worker for WorkflowWorker {
    type Key = ();
    type Message = WorkflowWorkerMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), WorkflowWorkerMessage>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    #[tracing::instrument(name = "workflow-worker", skip(self, _wid, _factory, msg, _state))]
    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), WorkflowWorkerMessage>>,
        Job { msg, .. }: Job<(), WorkflowWorkerMessage>,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match msg {
            WorkflowWorkerMessage::Execute { event_id, workflow } => {
                timed_async(|| Self::execute_workflow(event_id, workflow)).await?;
            }
        }

        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        _myself: ractor::ActorCell,
        message: ractor::SupervisionEvent,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match &message {
            ractor::SupervisionEvent::ActorTerminated(who, _, _)
            | ractor::SupervisionEvent::ActorFailed(who, _) => {
                tracing::error!("actor {who:?} failed");
                if let ractor::SupervisionEvent::ActorFailed(_, panic) = &message {
                    tracing::error!("panic: {panic}");
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct WorkflowWorkerBuilder {}
impl WorkerBuilder<WorkflowWorker, ()> for WorkflowWorkerBuilder {
    fn build(&mut self, _wid: usize) -> (WorkflowWorker, ()) {
        (WorkflowWorker {}, ())
    }
}
