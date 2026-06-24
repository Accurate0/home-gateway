//! Event dispatcher: the single subscriber that turns bus events into workflow
//! runs.
//!
//! It subscribes to the in-memory [`EventBus`](crate::event_bus::EventBus),
//! matches each [`EventBusMessage`] against the configured `workflows:`, gates on
//! the optional `when` condition, and forwards matching workflows to the
//! `WorkflowWorker` factory. It deliberately does **no** workflow execution
//! itself — it only matches and dispatches, so the factory's worker pool keeps
//! providing the parallelism.

use std::collections::HashMap;

use ractor::{
    Actor, ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, JobOptions},
};
use tokio::sync::broadcast::error::RecvError;
use tracing::Instrument;

use crate::{
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage, conditions},
    event_bus::{EventBusMessage, SensorMetric},
    settings::{TriggerMatcher, Workflow},
    types::SharedActorState,
};

pub struct WorkflowDispatcher {
    pub shared_actor_state: SharedActorState,
}

#[derive(Default)]
pub struct WorkflowDispatcherState {
    /// `(trigger name, sensor, metric) -> comparison satisfied at last reading`.
    /// Lets environment triggers fire on the rising edge only, matching the old
    /// plant-sensor semantics.
    last_satisfied: HashMap<(String, String, SensorMetric), bool>,
}

impl WorkflowDispatcher {
    pub const NAME: &str = "workflow-dispatcher";

    /// Decide whether `trigger.on` matches `msg`. The matcher's device/sensor
    /// references are registry ids, resolved to addresses here to compare against
    /// the event (which carries addresses). For environment triggers this also
    /// updates rising-edge state, so it takes `&mut state`.
    fn matches(
        &self,
        workflow: &Workflow,
        msg: &EventBusMessage,
        state: &mut WorkflowDispatcherState,
    ) -> bool {
        let Some(on) = workflow.on() else {
            return false;
        };
        let devices = &self.shared_actor_state.devices;
        match (on, msg) {
            (
                TriggerMatcher::Presence { sensor, present },
                EventBusMessage::Presence {
                    sensor: s,
                    present: p,
                    ..
                },
            ) => devices.address_or_self(sensor) == s.as_str() && present == p,
            (
                TriggerMatcher::Door { ieee_addr, open },
                EventBusMessage::Door {
                    ieee_addr: a,
                    open: o,
                    ..
                },
            ) => devices.address_or_self(ieee_addr) == a.as_str() && open == o,
            (
                TriggerMatcher::Switch { ieee_addr, action },
                EventBusMessage::SwitchAction {
                    ieee_addr: a,
                    action: ac,
                    ..
                },
            ) => devices.address_or_self(ieee_addr) == a.as_str() && action == ac,
            (
                TriggerMatcher::Environment {
                    sensor,
                    metric,
                    cmp,
                },
                EventBusMessage::Environment {
                    sensor: s, reading, ..
                },
            ) => {
                if devices.address_or_self(sensor) != s.as_str() || *metric != reading.metric() {
                    return false;
                }
                let satisfied = cmp.matches(reading.value());
                let key = (workflow.name.clone(), s.clone(), reading.metric());
                let was_satisfied = state.last_satisfied.insert(key, satisfied).unwrap_or(false);
                // rising edge only: fire when the threshold is newly crossed
                satisfied && !was_satisfied
            }
            (TriggerMatcher::Cron { .. }, EventBusMessage::Cron { name, .. }) => {
                &workflow.name == name
            }
            _ => false,
        }
    }

    async fn handle(
        &self,
        msg: EventBusMessage,
        state: &mut WorkflowDispatcherState,
    ) -> Result<(), ActorProcessingErr> {
        let event_id = msg.event_id();
        crate::metrics::record_event(msg.kind());
        let settings = self.shared_actor_state.settings.clone();

        for workflow in settings.workflows.values() {
            if !workflow.enabled || !self.matches(workflow, &msg, state) {
                continue;
            }

            let trigger_span = tracing::info_span!(
                "trigger.evaluate",
                otel.name = format!("trigger: {}", workflow.name),
                trigger = workflow.name,
                event_kind = msg.kind(),
            );
            self.evaluate_trigger(event_id, workflow)
                .instrument(trigger_span)
                .await?;
        }

        Ok(())
    }

    /// Evaluate a single matched trigger: gate on `when`, honour the cooldown,
    /// and dispatch its workflow. Recorded as one `trigger.evaluate` span by the
    /// caller via [`Instrument`].
    async fn evaluate_trigger(
        &self,
        event_id: uuid::Uuid,
        workflow: &Workflow,
    ) -> Result<(), ActorProcessingErr> {
        if let Some(when) = workflow.when() {
            match conditions::eval(&self.shared_actor_state, when).await {
                Ok(true) => {}
                Ok(false) => {
                    tracing::info!(
                        "[{event_id}] trigger '{}' matched but `when` not satisfied",
                        workflow.name
                    );
                    crate::metrics::record_trigger(
                        &workflow.name,
                        crate::metrics::TriggerOutcome::WhenNotMet,
                    );
                    return Ok(());
                }
                Err(e) => {
                    tracing::error!(
                        "[{event_id}] trigger '{}' `when` evaluation failed: {e}",
                        workflow.name
                    );
                    crate::metrics::record_trigger(
                        &workflow.name,
                        crate::metrics::TriggerOutcome::WhenError,
                    );
                    return Ok(());
                }
            }
        }

        if let Some(cooldown) = workflow.cooldown()
            && !self.cooldown_ok(&workflow.name, cooldown).await?
        {
            tracing::info!(
                "[{event_id}] trigger '{}' within cooldown, skipping",
                workflow.name
            );
            crate::metrics::record_trigger(
                &workflow.name,
                crate::metrics::TriggerOutcome::CooldownSkipped,
            );
            return Ok(());
        }

        tracing::info!("[{event_id}] trigger '{}' fired", workflow.name);
        crate::metrics::record_trigger(&workflow.name, crate::metrics::TriggerOutcome::Fired);
        self.dispatch_workflow(event_id, workflow.clone())?;

        Ok(())
    }

    /// Returns `true` if the trigger is allowed to fire now (no record, or the
    /// cooldown has elapsed), recording the firing time. Backed by the
    /// `trigger_cooldowns` table so the window survives restarts.
    async fn cooldown_ok(
        &self,
        name: &str,
        cooldown: chrono::TimeDelta,
    ) -> Result<bool, ActorProcessingErr> {
        let now = chrono::Utc::now();
        let db = &self.shared_actor_state.db;

        let last = sqlx::query!(
            "SELECT last_fired FROM trigger_cooldowns WHERE name = $1",
            name
        )
        .fetch_optional(db)
        .await?;

        if let Some(row) = last
            && now - row.last_fired < cooldown
        {
            return Ok(false);
        }

        sqlx::query!(
            "INSERT INTO trigger_cooldowns (name, last_fired) VALUES ($1, $2) \
             ON CONFLICT (name) DO UPDATE SET last_fired = EXCLUDED.last_fired",
            name,
            now
        )
        .execute(db)
        .await?;

        Ok(true)
    }

    fn dispatch_workflow(
        &self,
        event_id: uuid::Uuid,
        workflow: Workflow,
    ) -> Result<(), ActorProcessingErr> {
        let Some(actor) = ractor::registry::where_is(WorkflowWorker::NAME.to_string()) else {
            tracing::warn!("[{event_id}] workflow factory not found, dropping trigger");
            return Ok(());
        };

        actor.send_message(FactoryMessage::Dispatch(Job {
            key: (),
            msg: WorkflowWorkerMessage::Execute { event_id, workflow },
            options: JobOptions::default(),
            accepted: None,
        }))?;

        Ok(())
    }
}

impl Actor for WorkflowDispatcher {
    type Msg = EventBusMessage;
    type State = WorkflowDispatcherState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        // bridge the broadcast receiver into this actor's mailbox so matching is
        // serialized through `handle` while execution fans out to the factory
        let mut rx = self.shared_actor_state.event_bus.subscribe();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        if myself.send_message(msg).is_err() {
                            // dispatcher stopped; nothing left to feed
                            break;
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        tracing::warn!("event dispatcher lagged, dropped {n} events");
                    }
                    Err(RecvError::Closed) => break,
                }
            }
        });

        Ok(WorkflowDispatcherState::default())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        // Root span per event: `parent: None` detaches from any span ractor
        // re-enters from the `send_message` call site, so each dispatched event
        // is its own trace rather than all sharing the bridge task's context.
        let span = tracing::info_span!(
            parent: None,
            "dispatch_event",
            event_kind = message.kind(),
            event_id = %message.event_id(),
        );

        if let Err(e) = WorkflowDispatcher::handle(self, message, state)
            .instrument(span)
            .await
        {
            tracing::error!("error while dispatching event: {e}");
        }

        Ok(())
    }
}
