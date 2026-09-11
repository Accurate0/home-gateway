use crate::actors::devices::robot_vacuum;
use crate::actors::system::push::types::{PushAction, PushActionKind};
use crate::actors::system::rpc;
use crate::actors::workflows::manager::WorkflowManager;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::mqtt::MqttClient;
use crate::settings::TemplateString;
use crate::settings::http_method::HttpMethod;
use crate::settings::vacuum_command::VacuumCommand;
use crate::settings::workflow_context::ContextSource;
use crate::{
    actors::devices::light::{LightHandler, LightHandlerMessage},
    actors::workflows::manager::WorkflowRun,
    event_bus::EventBusMessage,
    integrations::notify::{Notification, notify},
    settings::workflow::{EnableState, LightState, Step, SwitchState},
    settings::{NotifyAction, NotifyActionKind, ReusableWorkflow, WorkflowDefinition},
    state::AppState,
    timer::timed_async,
};
use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use reqwest_middleware::ClientWithMiddleware;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::time::Duration;
use tracing::Instrument;
use uuid::Uuid;

pub mod conditions;
pub mod context;
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
    #[error("switch `{0}` has no control path: only a switch declared `as: light` can be driven")]
    NotAControllableSwitch(String),
    #[error("home assistant is not configured")]
    HomeAssistantNotConfigured,
    #[error("workflow context `{0}` is unavailable")]
    ContextUnavailable(&'static str),
    #[error("`{0}` is not a robot vacuum")]
    NotARobotVacuum(String),
    #[error("http request to {url} returned {status}")]
    Http {
        url: String,
        status: reqwest::StatusCode,
    },
    #[error(transparent)]
    HttpRequest(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Mqtt(#[from] crate::integrations::mqtt::MqttError),
    #[error(transparent)]
    HomeAssistant(#[from] crate::integrations::home_assistant::HomeAssistantError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Per-execution context threaded through the recursive executor.
#[derive(Clone, Copy)]
struct WorkflowContext<'a> {
    event_id: Uuid,
    depth: u8,
    dry_run: bool,
    /// Slug of the workflow that was triggered, preserved across `run_workflow`
    /// nesting so a `set_workflows_enabled` step never disables its own origin.
    origin_slug: &'a str,
    /// Template variables carried from the triggering event, substituted into
    /// `notify` messages.
    vars: &'a HashMap<String, String>,
    sources: &'a [ContextSource],
}

pub enum WorkflowWorkerMessage {
    Execute {
        event_id: Uuid,
        workflow: ReusableWorkflow,
        vars: HashMap<String, String>,
        traceparent: crate::tracing_context::TraceParent,
    },
}

pub struct WorkflowWorker {
    shared_actor_state: AppState,
}

impl WorkflowWorker {
    pub const NAME: &str = "workflow";

    pub async fn execute_workflow(
        &self,
        event_id: Uuid,
        workflow: ReusableWorkflow,
        vars: &HashMap<String, String>,
    ) -> Result<(), WorkflowError> {
        if !self
            .shared_actor_state
            .handles
            .expect::<WorkflowManager>()
            .enabled(&workflow.slug, workflow.enabled)
            .await
        {
            tracing::warn!("[{event_id}] workflow not executed as it's disabled");
            crate::metrics::record_workflow("disabled", Duration::ZERO);
            self.shared_actor_state
                .handles
                .expect::<WorkflowManager>()
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
        let start = std::time::Instant::now();
        let result = match context::resolve(&self.shared_actor_state, &workflow.context, vars).await
        {
            Ok(vars) => {
                let ctx = WorkflowContext {
                    event_id,
                    depth: 0,
                    dry_run: workflow.dry_run,
                    origin_slug: &workflow.slug,
                    vars: &vars,
                    sources: &workflow.context,
                };

                self.run_steps(ctx, &workflow.run).await
            }
            Err(e) => Err(e),
        };
        let elapsed = start.elapsed();
        let outcome = if result.is_ok() { "success" } else { "error" };
        crate::metrics::record_workflow(outcome, elapsed);
        self.shared_actor_state
            .handles
            .expect::<WorkflowManager>()
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

    async fn run_steps(
        &self,
        ctx: WorkflowContext<'_>,
        steps: &[Step],
    ) -> Result<(), WorkflowError> {
        for step in steps {
            self.run_step(ctx, step).await?;
        }
        Ok(())
    }

    async fn run_step(&self, ctx: WorkflowContext<'_>, step: &Step) -> Result<(), WorkflowError> {
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

    async fn dispatch_step(
        &self,
        ctx: WorkflowContext<'_>,
        step: &Step,
    ) -> Result<(), WorkflowError> {
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
            Step::Switch {
                ieee_addr, state, ..
            } => self.run_switch(ieee_addr.clone(), *state).await,
            Step::Scene { run, .. } => Box::pin(self.run_steps(ctx, run)).await,
            Step::Notify {
                notify: n,
                message,
                title,
                category,
                actions,
                acknowledge,
                ..
            } => {
                let notification = Notification::new(
                    message.render(ctx.vars),
                    *category,
                    format!("workflow:{}", ctx.origin_slug),
                )
                .with_actions(self.resolve_push_actions(actions))
                .with_acknowledge(*acknowledge);

                let notification = match title {
                    Some(title) => notification.with_title(title.render(ctx.vars)),
                    None => notification,
                };

                notify(std::slice::from_ref(n), notification);
                Ok(())
            }
            Step::Delay { seconds, .. } => {
                tokio::time::sleep(Duration::from_secs(*seconds)).await;
                Ok(())
            }
            Step::RunWorkflow { workflow, .. } => self.run_named_workflow(ctx, workflow).await,
            Step::SetMode { mode, .. } => self.run_set_mode(*mode).await,
            Step::SetWorkflowsEnabled { tag, state, .. } => {
                self.run_set_workflows_enabled(ctx, tag, *state).await
            }
            Step::HomeAssistant {
                call_service, data, ..
            } => self.run_home_assistant(call_service, data.clone()).await,
            Step::MqttPublish {
                topic,
                payload,
                retain,
                ..
            } => self.run_mqtt_publish(ctx, topic, payload, *retain).await,
            Step::Http {
                method,
                url,
                headers,
                body,
                ..
            } => {
                self.run_http(ctx, *method, url, headers, body.as_ref())
                    .await
            }
            Step::RobotVacuum {
                ieee_addr, command, ..
            } => self.run_robot_vacuum(ieee_addr, *command).await,
        }
    }

    fn resolve_push_actions(&self, actions: &[NotifyAction]) -> Vec<PushAction> {
        let settings = &self.shared_actor_state.settings;

        actions
            .iter()
            .filter_map(|action| {
                let kind = match &action.action {
                    NotifyActionKind::RunWorkflow { workflow } => {
                        let target = settings
                            .workflows
                            .get(workflow)
                            .map(WorkflowDefinition::body)?;
                        PushActionKind::RunWorkflow {
                            slug: target.slug.clone(),
                        }
                    }
                    NotifyActionKind::Snooze { seconds } => {
                        PushActionKind::Snooze { seconds: *seconds }
                    }
                    NotifyActionKind::Dismiss => PushActionKind::Dismiss,
                    NotifyActionKind::Acknowledge => PushActionKind::Acknowledge,
                };

                Some(PushAction {
                    label: action.label.clone(),
                    kind,
                })
            })
            .collect()
    }

    async fn run_home_assistant(
        &self,
        call_service: &str,
        data: serde_json::Value,
    ) -> Result<(), WorkflowError> {
        let home_assistant = self
            .shared_actor_state
            .handles
            .get::<HomeAssistant>()
            .ok_or(WorkflowError::HomeAssistantNotConfigured)?;

        let (domain, service) = call_service.split_once('.').ok_or_else(|| {
            WorkflowError::HomeAssistant(
                crate::integrations::home_assistant::HomeAssistantError::InvalidService(
                    call_service.to_owned(),
                ),
            )
        })?;

        home_assistant.call_service(domain, service, data).await?;
        Ok(())
    }

    async fn run_mqtt_publish(
        &self,
        ctx: WorkflowContext<'_>,
        topic: &TemplateString,
        payload: &TemplateString,
        retain: bool,
    ) -> Result<(), WorkflowError> {
        let topic = topic.render(ctx.vars);

        self.shared_actor_state
            .handles
            .expect::<MqttClient>()
            .send_event_raw(topic.clone(), &payload.render(ctx.vars), retain)
            .await?;

        tracing::info!("[{}] published to {topic}", ctx.event_id);
        Ok(())
    }

    async fn run_http(
        &self,
        ctx: WorkflowContext<'_>,
        method: HttpMethod,
        url: &TemplateString,
        headers: &BTreeMap<String, TemplateString>,
        body: Option<&TemplateString>,
    ) -> Result<(), WorkflowError> {
        let url = url.render(ctx.vars);
        let client = self
            .shared_actor_state
            .handles
            .expect::<ClientWithMiddleware>();

        let mut request = client.request(method.as_reqwest(), &url);

        for (name, value) in headers {
            request = request.header(name.as_str(), value.render(ctx.vars));
        }

        if let Some(body) = body {
            request = request.body(body.render(ctx.vars));
        }

        let status = request.send().await?.status();

        if !status.is_success() {
            return Err(WorkflowError::Http { url, status });
        }

        tracing::info!("[{}] http {method:?} {url} returned {status}", ctx.event_id);
        Ok(())
    }

    async fn run_robot_vacuum(
        &self,
        device: &str,
        command: VacuumCommand,
    ) -> Result<(), WorkflowError> {
        let registry = &self.shared_actor_state.devices;
        let address = registry.address_or_self(device);

        if let Some(settings) = registry.roborock(address) {
            let home_assistant = self
                .shared_actor_state
                .handles
                .get::<HomeAssistant>()
                .ok_or(WorkflowError::HomeAssistantNotConfigured)?;

            robot_vacuum::command::roborock(home_assistant, settings, command).await?;
            return Ok(());
        }

        if let Some(settings) = registry.valetudo(address) {
            let mqtt = self.shared_actor_state.handles.expect::<MqttClient>();

            robot_vacuum::command::valetudo(mqtt, settings, command).await?;
            return Ok(());
        }

        Err(WorkflowError::NotARobotVacuum(device.to_owned()))
    }

    /// Enable/disable every workflow carrying `tag`, skipping the workflow the
    /// step originated from so a switch can always undo itself. `Toggle` reads
    /// the current state of the set first and flips it as a unit.
    async fn run_set_workflows_enabled(
        &self,
        ctx: WorkflowContext<'_>,
        tag: &str,
        state: EnableState,
    ) -> Result<(), WorkflowError> {
        let settings = self.shared_actor_state.settings.clone();
        let manager = self.shared_actor_state.handles.expect::<WorkflowManager>();

        let targets = settings
            .workflows
            .values()
            .map(WorkflowDefinition::body)
            .filter(|w| w.tags.iter().any(|t| t == tag) && w.slug != ctx.origin_slug)
            .collect::<Vec<_>>();

        if targets.is_empty() {
            tracing::warn!("[{}] no workflows tagged `{tag}`", ctx.event_id);
            return Ok(());
        }

        let enabled = match state {
            EnableState::Enabled => true,
            EnableState::Disabled => false,
            EnableState::Toggle => {
                let mut any_enabled = false;
                for workflow in &targets {
                    if manager.enabled(&workflow.slug, workflow.enabled).await {
                        any_enabled = true;
                        break;
                    }
                }
                !any_enabled
            }
        };

        for workflow in targets {
            manager
                .set_enabled(&workflow.slug, enabled)
                .await
                .map_err(|e| WorkflowError::Other(e.into()))?;
        }

        tracing::info!(
            "[{}] set workflows tagged `{tag}` to {enabled}",
            ctx.event_id
        );
        Ok(())
    }

    async fn run_set_mode(&self, mode: crate::mode::Mode) -> Result<(), WorkflowError> {
        let previous = self
            .shared_actor_state
            .handles
            .expect::<WorkflowManager>()
            .set_mode(mode)
            .await
            .map_err(|e| WorkflowError::Other(e.into()))?;

        if let Some(previous) = previous {
            self.shared_actor_state
                .event_bus
                .publish(EventBusMessage::Mode {
                    event_id: Uuid::new_v4(),
                    mode,
                    previous,
                });
        }

        Ok(())
    }

    /// Expand a `run_workflow` step: look the named workflow up in settings and
    /// run its steps with an incremented depth. `MAX_DEPTH` bounds transitive
    /// self-reference so a workflow cycle can't recurse forever.
    async fn run_named_workflow(
        &self,
        ctx: WorkflowContext<'_>,
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
        let Some(workflow) = settings.workflows.get(name).map(WorkflowDefinition::body) else {
            tracing::warn!(
                "[{}] run_workflow references unknown workflow `{name}`",
                ctx.event_id
            );
            return Ok(());
        };

        if !self
            .shared_actor_state
            .handles
            .expect::<WorkflowManager>()
            .enabled(&workflow.slug, workflow.enabled)
            .await
        {
            tracing::info!("[{}] skipping disabled workflow `{name}`", ctx.event_id);
            return Ok(());
        }

        let missing: Vec<ContextSource> = workflow
            .context
            .iter()
            .copied()
            .filter(|source| !ctx.sources.contains(source))
            .collect();

        let vars = context::resolve(&self.shared_actor_state, &missing, ctx.vars).await?;
        let sources: Vec<ContextSource> = ctx.sources.iter().chain(&missing).copied().collect();

        let child = WorkflowContext {
            event_id: ctx.event_id,
            depth: ctx.depth + 1,
            dry_run: ctx.dry_run || workflow.dry_run,
            origin_slug: ctx.origin_slug,
            vars: &vars,
            sources: &sources,
        };
        Box::pin(self.run_steps(child, &workflow.run)).await
    }

    async fn run_light(&self, device: String, state: LightState) -> Result<(), WorkflowError> {
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

        rpc::cast_factory(LightHandler::NAME, light_actor_message)
            .map_err(|e| WorkflowError::Messaging(e.to_string()))
    }

    async fn run_switch(&self, device: String, state: SwitchState) -> Result<(), WorkflowError> {
        let ieee_addr = self
            .shared_actor_state
            .devices
            .address_or_self(&device)
            .to_owned();

        if self.shared_actor_state.devices.light(&ieee_addr).is_none() {
            return Err(WorkflowError::NotAControllableSwitch(ieee_addr));
        }

        let light_state = match state {
            SwitchState::On => LightState::On,
            SwitchState::Off => LightState::Off,
            SwitchState::Toggle => LightState::Toggle,
        };

        self.run_light(ieee_addr, light_state).await
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

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), WorkflowWorkerMessage>>,
        Job { msg, .. }: Job<(), WorkflowWorkerMessage>,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match msg {
            WorkflowWorkerMessage::Execute {
                event_id,
                workflow,
                vars,
                traceparent,
            } => {
                let span = tracing::info_span!(
                    parent: None,
                    "workflow-worker",
                    workflow = workflow.name,
                    event_id = %event_id,
                );
                crate::tracing_context::set_parent(&span, traceparent.as_deref());

                let result = timed_async(|| async {
                    self.execute_workflow(event_id, workflow, &vars)
                        .await
                        .map_err(anyhow::Error::from)
                })
                .instrument(span)
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
    pub shared_actor_state: AppState,
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
