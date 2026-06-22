use crate::{
    actors::{
        devices::environment_sensor::{
            EnvironmentSensorHandler, LatestReading, Message as EnvironmentMessage,
        },
        events::door_events::{DerivedDoorEvents, DoorEventsMessage},
        light::{LightHandler, LightHandlerMessage},
        vacuum::{VacuumActor, VacuumMessage},
    },
    notify::notify,
    settings::workflow::{
        Comparison, Condition, EnvMetric, LightState, Step, VacuumCommand, WorkflowSettings,
    },
    timer::timed_async,
    types::SharedActorState,
    types::db::DoorState,
};
use chrono::Local;
use ractor::{
    ActorRef, RpcReplyPort,
    factory::{FactoryMessage, Job, JobOptions, Worker, WorkerBuilder, WorkerId},
};
use std::time::Duration;
use uuid::Uuid;

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
struct ExecCtx {
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
            return Ok(());
        }

        tracing::info!("executing workflow for: {event_id}");
        let ctx = ExecCtx { event_id, depth: 0 };
        self.run_steps(ctx, &workflow.run).await
    }

    async fn run_steps(&self, ctx: ExecCtx, steps: &[Step]) -> Result<(), WorkflowError> {
        for step in steps {
            self.run_step(ctx, step).await?;
        }
        Ok(())
    }

    async fn run_step(&self, ctx: ExecCtx, step: &Step) -> Result<(), WorkflowError> {
        // a failed guard skips only this step, not the rest of the workflow
        if let Some(when) = step.guard()
            && !self.eval(ctx, when).await?
        {
            tracing::info!("[{}] skipping step, guard not satisfied", ctx.event_id);
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
            Step::Vacuum { command, .. } => self.run_vacuum(*command),
            Step::Scene { run, .. } => Box::pin(self.run_steps(ctx, run)).await,
            Step::Notify { notify: n, message, .. } => {
                notify(&[n.clone()], message.clone());
                Ok(())
            }
            Step::Delay { seconds, .. } => {
                tokio::time::sleep(Duration::from_secs(*seconds)).await;
                Ok(())
            }
            Step::RunWorkflow { workflow, .. } => {
                // implemented alongside named workflows in a later phase
                let _ = workflow;
                Err(WorkflowError::NotImplemented("run_workflow action"))
            }
        }
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

    async fn run_light(
        &self,
        ieee_addr: String,
        state: LightState,
    ) -> Result<(), WorkflowError> {
        let actor = ractor::registry::where_is(LightHandler::NAME.to_string())
            .ok_or(WorkflowError::ActorNotFound(LightHandler::NAME))?;

        let light_actor_message = match state {
            LightState::On => LightHandlerMessage::TurnOn { ieee_addr },
            LightState::Off => LightHandlerMessage::TurnOff { ieee_addr },
            LightState::Toggle => LightHandlerMessage::Toggle { ieee_addr },
            LightState::SetBrightness { value } => {
                LightHandlerMessage::SetBrightness { ieee_addr, value }
            }
            LightState::IncreaseBrightness { value, on_off } => LightHandlerMessage::BrightnessMove {
                ieee_addr,
                value: value.try_into().map_err(anyhow::Error::from)?,
                on_off,
            },
            LightState::DecreaseBrightness { value, on_off } => LightHandlerMessage::BrightnessMove {
                ieee_addr,
                value: -TryInto::<i64>::try_into(value).map_err(anyhow::Error::from)?,
                on_off,
            },
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

    async fn eval(&self, ctx: ExecCtx, cond: &Condition) -> Result<bool, WorkflowError> {
        match cond {
            Condition::Light { ieee_addr, on } => Ok(self.query_light_on(ieee_addr).await? == *on),
            Condition::Environment {
                sensor,
                metric,
                cmp,
            } => self.eval_environment(sensor, *metric, *cmp).await,
            Condition::Door { ieee_addr, open } => Ok(self.query_door_open(ieee_addr).await? == *open),
            Condition::Presence { sensor, .. } => {
                let _ = sensor;
                Err(WorkflowError::NotImplemented("presence condition"))
            }
            Condition::TimeOfDay { after, before } => {
                let now = Local::now().time();
                Ok(match (after, before) {
                    (Some(a), Some(b)) if a > b => now >= *a || now < *b, // wraps midnight
                    (Some(a), Some(b)) => now >= *a && now < *b,
                    (Some(a), None) => now >= *a,
                    (None, Some(b)) => now < *b,
                    (None, None) => true,
                })
            }
            Condition::All { conditions } => {
                for c in conditions {
                    if !Box::pin(self.eval(ctx, c)).await? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Any { conditions } => {
                for c in conditions {
                    if Box::pin(self.eval(ctx, c)).await? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Not { condition } => Ok(!Box::pin(self.eval(ctx, condition)).await?),
        }
    }

    async fn query_light_on(&self, ieee_addr: &str) -> Result<bool, WorkflowError> {
        let actor = ractor::registry::where_is(LightHandler::NAME.to_string())
            .ok_or(WorkflowError::ActorNotFound(LightHandler::NAME))?;

        let (tx, rx) = ractor::concurrency::oneshot();
        let port: RpcReplyPort<bool> = (tx, Duration::from_secs(10)).into();
        let message = FactoryMessage::Dispatch(Job {
            key: (),
            msg: LightHandlerMessage::QueryPowerState {
                ieee_addr: ieee_addr.to_owned(),
                reply: port,
            },
            options: JobOptions::default(),
            accepted: None,
        });

        actor
            .send_message(message)
            .map_err(|e| WorkflowError::Messaging(e.to_string()))?;

        rx.await.map_err(|e| WorkflowError::Messaging(e.to_string()))
    }

    async fn eval_environment(
        &self,
        sensor: &str,
        metric: EnvMetric,
        cmp: Comparison,
    ) -> Result<bool, WorkflowError> {
        let actor = ractor::registry::where_is(EnvironmentSensorHandler::NAME.to_string())
            .ok_or(WorkflowError::ActorNotFound(EnvironmentSensorHandler::NAME))?;

        let (tx, rx) = ractor::concurrency::oneshot();
        let port: RpcReplyPort<Option<LatestReading>> = (tx, Duration::from_secs(10)).into();
        let message = FactoryMessage::Dispatch(Job {
            key: (),
            msg: EnvironmentMessage::QueryLatest {
                entity_id: sensor.to_owned(),
                reply: port,
            },
            options: JobOptions::default(),
            accepted: None,
        });
        actor
            .send_message(message)
            .map_err(|e| WorkflowError::Messaging(e.to_string()))?;

        let Some(reading) = rx.await.map_err(|e| WorkflowError::Messaging(e.to_string()))? else {
            tracing::warn!("no readings for environment sensor {sensor}");
            return Ok(false);
        };

        let value = match metric {
            EnvMetric::Temperature => Some(reading.temperature),
            EnvMetric::Humidity => reading.humidity,
            EnvMetric::Pressure => reading.pressure,
            EnvMetric::Lux => reading.lux,
            EnvMetric::UvIndex => reading.uv_index,
        };

        let Some(value) = value else {
            tracing::warn!("environment sensor {sensor} has no reading for {metric:?}");
            return Ok(false);
        };

        Ok(cmp.matches(value))
    }

    async fn query_door_open(&self, ieee_addr: &str) -> Result<bool, WorkflowError> {
        let actor = ractor::registry::where_is(DerivedDoorEvents::NAME.to_string())
            .ok_or(WorkflowError::ActorNotFound(DerivedDoorEvents::NAME))?;

        let (tx, rx) = ractor::concurrency::oneshot();
        let port: RpcReplyPort<Option<DoorState>> = (tx, Duration::from_secs(10)).into();
        actor
            .send_message(DoorEventsMessage::QueryState {
                ieee_addr: ieee_addr.to_owned(),
                reply: port,
            })
            .map_err(|e| WorkflowError::Messaging(e.to_string()))?;

        match rx.await.map_err(|e| WorkflowError::Messaging(e.to_string()))? {
            Some(state) => Ok(matches!(state, DoorState::Open)),
            None => {
                tracing::warn!("no door state for {ieee_addr}");
                Ok(false)
            }
        }
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
