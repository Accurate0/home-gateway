use crate::{
    actors::{
        light::{LightHandler, LightHandlerMessage},
        vacuum::{VacuumActor, VacuumMessage},
    },
    notify::notify,
    settings::workflow::{LightState, Step, VacuumCommand, WorkflowSettings},
    timer::timed_async,
    types::SharedActorState,
};
use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, JobOptions, Worker, WorkerBuilder, WorkerId},
};
use std::time::Duration;
use tracing::Instrument;
use uuid::Uuid;

pub mod conditions;
pub mod spawn;

/// Maximum nesting depth for `run_workflow` expansion, guarding against
/// workflows that (directly or transitively) reference themselves.
const MAX_DEPTH: u8 = 8;

#[derive(thiserror::Error, Debug)]
pub enum WorkflowError {
    #[error("actor `{0}` not found")]
    ActorNotFound(&'static str),
    #[error("workflow recursion depth exceeded (>{MAX_DEPTH})")]
    DepthExceeded,
    #[error("messaging error: {0}")]
    Messaging(String),
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Per-execution context threaded through the recursive executor.
#[derive(Clone, Copy)]
struct WorkflowContext {
    event_id: Uuid,
    depth: u8,
}

pub enum WorkflowWorkerMessage {
    Execute {
        event_id: Uuid,
        workflow: WorkflowSettings,
    },
}

pub struct WorkflowWorker {
    shared_actor_state: SharedActorState,
}

impl WorkflowWorker {
    pub const NAME: &str = "workflow";

    pub async fn execute_workflow(
        &self,
        event_id: Uuid,
        workflow: WorkflowSettings,
    ) -> Result<(), WorkflowError> {
        if !workflow.enabled {
            tracing::warn!("[{event_id}] workflow not executed as it's disabled in config");
            crate::metrics::record_workflow("disabled", Duration::ZERO);
            return Ok(());
        }

        tracing::info!("executing workflow for: {event_id}");
        let ctx = WorkflowContext { event_id, depth: 0 };
        let start = std::time::Instant::now();
        let result = self.run_steps(ctx, &workflow.run).await;
        let outcome = if result.is_ok() { "success" } else { "error" };
        crate::metrics::record_workflow(outcome, start.elapsed());
        result
    }

    async fn run_steps(&self, ctx: WorkflowContext, steps: &[Step]) -> Result<(), WorkflowError> {
        for step in steps {
            self.run_step(ctx, step).await?;
        }
        Ok(())
    }

    async fn run_step(&self, ctx: WorkflowContext, step: &Step) -> Result<(), WorkflowError> {
        // a failed guard skips only this step, not the rest of the workflow
        if let Some(when) = step.guard()
            && !conditions::eval(&self.shared_actor_state, when).await?
        {
            tracing::info!("[{}] skipping step, guard not satisfied", ctx.event_id);
            return Ok(());
        }

        let span = tracing::info_span!(
            "step.execute",
            otel.name = format!("step: {}", step.kind()),
            step = step.kind(),
            event_id = %ctx.event_id,
        );
        let start = std::time::Instant::now();
        let result = self.dispatch_step(ctx, step).instrument(span).await;
        crate::metrics::record_step(step.kind(), result.is_ok(), start.elapsed());
        result
    }

    async fn dispatch_step(&self, ctx: WorkflowContext, step: &Step) -> Result<(), WorkflowError> {
        match step {
            Step::Light {
                ieee_addr, state, ..
            } => self.run_light(ieee_addr.clone(), state.clone()).await,
            Step::Switch { ieee_addr, .. } => {
                // implemented in a later phase (needs a smart-switch control path)
                let _ = ieee_addr;
                Err(WorkflowError::NotImplemented("switch action"))
            }
            Step::Vacuum { command, .. } => self.run_vacuum(*command),
            Step::Scene { run, .. } => Box::pin(self.run_steps(ctx, run)).await,
            Step::Notify {
                notify: n, message, ..
            } => {
                notify(std::slice::from_ref(n), message.clone());
                Ok(())
            }
            Step::Delay { seconds, .. } => {
                tokio::time::sleep(Duration::from_secs(*seconds)).await;
                Ok(())
            }
            Step::RunWorkflow { workflow, .. } => self.run_named_workflow(ctx, workflow).await,
        }
    }

    /// Expand a `run_workflow` step: look the named workflow up in settings and
    /// run its steps with an incremented depth. `MAX_DEPTH` bounds transitive
    /// self-reference so a workflow cycle can't recurse forever.
    async fn run_named_workflow(
        &self,
        ctx: WorkflowContext,
        name: &str,
    ) -> Result<(), WorkflowError> {
        if ctx.depth >= MAX_DEPTH {
            tracing::error!(
                "[{}] workflow recursion depth exceeded at `{name}`",
                ctx.event_id
            );
            return Err(WorkflowError::DepthExceeded);
        }

        let settings = self.shared_actor_state.settings.load_full();
        let Some(workflow) = settings.workflows.get(name) else {
            tracing::warn!(
                "[{}] run_workflow references unknown workflow `{name}`",
                ctx.event_id
            );
            return Ok(());
        };

        if !workflow.enabled {
            tracing::info!("[{}] skipping disabled workflow `{name}`", ctx.event_id);
            return Ok(());
        }

        let child = WorkflowContext {
            event_id: ctx.event_id,
            depth: ctx.depth + 1,
        };
        Box::pin(self.run_steps(child, &workflow.run)).await
    }

    fn run_vacuum(&self, command: VacuumCommand) -> Result<(), WorkflowError> {
        let actor = ractor::registry::where_is(VacuumActor::NAME.to_string())
            .ok_or(WorkflowError::ActorNotFound(VacuumActor::NAME))?;
        let message = match command {
            VacuumCommand::Start => VacuumMessage::Start,
            VacuumCommand::Stop => VacuumMessage::Stop,
            VacuumCommand::Pause => VacuumMessage::Pause,
            VacuumCommand::Home => VacuumMessage::Home,
        };
        actor
            .send_message(message)
            .map_err(|e| WorkflowError::Messaging(e.to_string()))
    }

    async fn run_light(&self, ieee_addr: String, state: LightState) -> Result<(), WorkflowError> {
        let actor = ractor::registry::where_is(LightHandler::NAME.to_string())
            .ok_or(WorkflowError::ActorNotFound(LightHandler::NAME))?;

        let light_actor_message = match state {
            LightState::On => LightHandlerMessage::TurnOn { ieee_addr },
            LightState::Off => LightHandlerMessage::TurnOff { ieee_addr },
            LightState::Toggle => LightHandlerMessage::Toggle { ieee_addr },
            LightState::SetBrightness { value } => {
                LightHandlerMessage::SetBrightness { ieee_addr, value }
            }
            LightState::IncreaseBrightness { value, on_off } => {
                LightHandlerMessage::BrightnessMove {
                    ieee_addr,
                    value: value.try_into().map_err(anyhow::Error::from)?,
                    on_off,
                }
            }
            LightState::DecreaseBrightness { value, on_off } => {
                LightHandlerMessage::BrightnessMove {
                    ieee_addr,
                    value: -TryInto::<i64>::try_into(value).map_err(anyhow::Error::from)?,
                    on_off,
                }
            }
            LightState::StopBrightness => LightHandlerMessage::BrightnessMove {
                ieee_addr,
                value: 0,
                on_off: false,
            },
            LightState::IncreaseColourTemperature { value } => {
                LightHandlerMessage::ColourTemperatureMove {
                    ieee_addr,
                    value: value.try_into().map_err(anyhow::Error::from)?,
                }
            }
            LightState::DecreaseColourTemperature { value } => {
                LightHandlerMessage::ColourTemperatureMove {
                    ieee_addr,
                    value: -TryInto::<i64>::try_into(value).map_err(anyhow::Error::from)?,
                }
            }
            LightState::StopColourTemperature => LightHandlerMessage::ColourTemperatureMove {
                ieee_addr,
                value: 0,
            },
        };

        let message = FactoryMessage::Dispatch(Job {
            key: (),
            msg: light_actor_message,
            options: JobOptions::default(),
            accepted: None,
        });
        actor
            .send_message(message)
            .map_err(|e| WorkflowError::Messaging(e.to_string()))
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
                let result = timed_async(|| async {
                    self.execute_workflow(event_id, workflow)
                        .await
                        .map_err(anyhow::Error::from)
                })
                .await;

                if let Err(e) = result {
                    tracing::error!("[{event_id}] workflow execution failed: {e}");
                }
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

pub struct WorkflowWorkerBuilder {
    pub shared_actor_state: SharedActorState,
}
impl WorkerBuilder<WorkflowWorker, ()> for WorkflowWorkerBuilder {
    fn build(&mut self, _wid: usize) -> (WorkflowWorker, ()) {
        (
            WorkflowWorker {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
