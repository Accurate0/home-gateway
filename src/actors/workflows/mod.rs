use crate::{
    actors::light::{LightHandler, LightHandlerMessage},
    actors::workflows::manager::WorkflowRun,
    event_bus::EventBusMessage,
    notify::notify,
    settings::workflow::{LightState, Step, Workflow},
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
pub mod dispatcher;
pub mod manager;
pub mod plan;
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
    #[error("home assistant is not configured")]
    HomeAssistantNotConfigured,
    #[error(transparent)]
    HomeAssistant(#[from] crate::home_assistant::HomeAssistantError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Per-execution context threaded through the recursive executor.
#[derive(Clone, Copy)]
struct WorkflowContext {
    event_id: Uuid,
    depth: u8,
    dry_run: bool,
}

pub enum WorkflowWorkerMessage {
    Execute { event_id: Uuid, workflow: Workflow },
}

pub struct WorkflowWorker {
    shared_actor_state: SharedActorState,
}

impl WorkflowWorker {
    pub const NAME: &str = "workflow";

    pub async fn execute_workflow(
        &self,
        event_id: Uuid,
        workflow: Workflow,
    ) -> Result<(), WorkflowError> {
        if !self
            .shared_actor_state
            .workflows
            .enabled(&workflow.slug, workflow.enabled)
            .await
        {
            tracing::warn!("[{event_id}] workflow not executed as it's disabled");
            crate::metrics::record_workflow("disabled", Duration::ZERO);
            self.shared_actor_state
                .workflows
                .record_run(WorkflowRun {
                    slug: workflow.slug.clone(),
                    name: workflow.name.clone(),
                    event_id,
                    outcome: "disabled".to_owned(),
                    dry_run: workflow.dry_run,
                    duration: Duration::ZERO,
                    error: None,
                })
                .await;
            return Ok(());
        }

        tracing::info!("executing workflow for: {event_id}");
        if workflow.dry_run {
            tracing::info!("[{event_id}] workflow running in dry-run (shadow) mode");
        }
        let ctx = WorkflowContext {
            event_id,
            depth: 0,
            dry_run: workflow.dry_run,
        };
        let start = std::time::Instant::now();
        let result = self.run_steps(ctx, &workflow.run).await;
        let elapsed = start.elapsed();
        let outcome = if result.is_ok() { "success" } else { "error" };
        crate::metrics::record_workflow(outcome, elapsed);
        self.shared_actor_state
            .workflows
            .record_run(WorkflowRun {
                slug: workflow.slug.clone(),
                name: workflow.name.clone(),
                event_id,
                outcome: outcome.to_owned(),
                dry_run: workflow.dry_run,
                duration: elapsed,
                error: result.as_ref().err().map(|e| e.to_string()),
            })
            .await;
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
        if ctx.dry_run
            && let Some(detail) = step.describe_action()
        {
            tracing::info!(
                "[{}] DRY-RUN would fire {}: {detail}",
                ctx.event_id,
                step.kind()
            );
            return Ok(());
        }

        match step {
            Step::Light {
                ieee_addr, state, ..
            } => self.run_light(ieee_addr.clone(), state.clone()).await,
            Step::Switch { ieee_addr, .. } => {
                // implemented in a later phase (needs a smart-switch control path)
                let _ = ieee_addr;
                Err(WorkflowError::NotImplemented("switch action"))
            }
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
            Step::SetMode { mode, active, .. } => self.run_set_mode(*mode, *active).await,
            Step::HomeAssistant {
                call_service, data, ..
            } => self.run_home_assistant(call_service, data.clone()).await,
        }
    }

    async fn run_home_assistant(
        &self,
        call_service: &str,
        data: serde_json::Value,
    ) -> Result<(), WorkflowError> {
        let home_assistant = self
            .shared_actor_state
            .home_assistant
            .as_ref()
            .ok_or(WorkflowError::HomeAssistantNotConfigured)?;

        let (domain, service) = call_service.split_once('.').ok_or_else(|| {
            WorkflowError::HomeAssistant(crate::home_assistant::HomeAssistantError::InvalidService(
                call_service.to_owned(),
            ))
        })?;

        home_assistant.call_service(domain, service, data).await?;
        Ok(())
    }

    async fn run_set_mode(
        &self,
        mode: crate::mode::Mode,
        active: bool,
    ) -> Result<(), WorkflowError> {
        let transitions = self
            .shared_actor_state
            .workflows
            .set_mode(mode, active)
            .await
            .map_err(|e| WorkflowError::Other(e.into()))?;

        for (mode, active) in transitions {
            self.shared_actor_state
                .event_bus
                .publish(EventBusMessage::Mode {
                    event_id: Uuid::new_v4(),
                    mode,
                    active,
                });
        }
        Ok(())
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

        let settings = self.shared_actor_state.settings.clone();
        let Some(workflow) = settings.workflows.get(name) else {
            tracing::warn!(
                "[{}] run_workflow references unknown workflow `{name}`",
                ctx.event_id
            );
            return Ok(());
        };

        if !self
            .shared_actor_state
            .workflows
            .enabled(&workflow.slug, workflow.enabled)
            .await
        {
            tracing::info!("[{}] skipping disabled workflow `{name}`", ctx.event_id);
            return Ok(());
        }

        let child = WorkflowContext {
            event_id: ctx.event_id,
            depth: ctx.depth + 1,
            dry_run: ctx.dry_run || workflow.dry_run,
        };
        Box::pin(self.run_steps(child, &workflow.run)).await
    }

    async fn run_light(&self, device: String, state: LightState) -> Result<(), WorkflowError> {
        let actor = ractor::registry::where_is(LightHandler::NAME.to_string())
            .ok_or(WorkflowError::ActorNotFound(LightHandler::NAME))?;

        let ieee_addr = self
            .shared_actor_state
            .devices
            .address_or_self(&device)
            .to_owned();

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
