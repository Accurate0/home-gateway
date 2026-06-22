//! Event dispatcher: the single subscriber that turns bus events into workflow
//! runs.
//!
//! It subscribes to the in-memory [`EventBus`](crate::event_bus::EventBus),
//! matches each [`EventBusMessage`] against the configured `triggers:`, gates on
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
    settings::{Trigger, TriggerMatcher, workflow::WorkflowSettings},
    types::SharedActorState,
};

pub struct EventDispatcher {
    pub shared_actor_state: SharedActorState,
}

#[derive(Default)]
pub struct EventDispatcherState {
    /// `(trigger name, sensor, metric) -> comparison satisfied at last reading`.
    /// Lets environment triggers fire on the rising edge only, matching the old
    /// plant-sensor semantics.
    last_satisfied: HashMap<(String, String, SensorMetric), bool>,
}

impl EventDispatcher {
    pub const NAME: &str = "event-dispatcher";

    /// Decide whether `trigger.on` matches `msg`. For environment triggers this
    /// also updates rising-edge state, so it takes `&mut state`.
    fn matches(trigger: &Trigger, msg: &EventBusMessage, state: &mut EventDispatcherState) -> bool {
        match (&trigger.on, msg) {
            (
                TriggerMatcher::Presence { sensor, present },
                EventBusMessage::Presence {
                    sensor: s,
                    present: p,
                    ..
                },
            ) => sensor == s && present == p,
            (
                TriggerMatcher::Door { ieee_addr, open },
                EventBusMessage::Door {
                    ieee_addr: a,
                    open: o,
                    ..
                },
            ) => ieee_addr == a && open == o,
            (
                TriggerMatcher::Switch { ieee_addr, action },
                EventBusMessage::SwitchAction {
                    ieee_addr: a,
                    action: ac,
                    ..
                },
            ) => ieee_addr == a && action == ac,
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
                if sensor != s || *metric != reading.metric() {
                    return false;
                }
                let satisfied = cmp.matches(reading.value());
                let key = (trigger.name.clone(), s.clone(), reading.metric());
                let was_satisfied = state.last_satisfied.insert(key, satisfied).unwrap_or(false);
                // rising edge only: fire when the threshold is newly crossed
                satisfied && !was_satisfied
            }
            _ => false,
        }
    }

    #[tracing::instrument(
        name = "dispatch_event",
        skip_all,
        fields(event_kind = msg.kind(), event_id = %msg.event_id()),
    )]
    async fn handle(
        &self,
        msg: EventBusMessage,
        state: &mut EventDispatcherState,
    ) -> Result<(), ActorProcessingErr> {
        let event_id = msg.event_id();
        crate::metrics::record_event(msg.kind());
        // owned snapshot so we don't hold the arc-swap guard across awaits
        let settings = self.shared_actor_state.settings.load_full();

        for trigger in &settings.triggers {
            if !trigger.enabled || !Self::matches(trigger, &msg, state) {
                continue;
            }

            let trigger_span = tracing::info_span!(
                "trigger.evaluate",
                otel.name = format!("trigger: {}", trigger.name),
                trigger = trigger.name,
                event_kind = msg.kind(),
            );
            self.evaluate_trigger(event_id, trigger)
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
        trigger: &Trigger,
    ) -> Result<(), ActorProcessingErr> {
        if let Some(when) = &trigger.when {
            match conditions::eval(&self.shared_actor_state, when).await {
                Ok(true) => {}
                Ok(false) => {
                    tracing::info!(
                        "[{event_id}] trigger '{}' matched but `when` not satisfied",
                        trigger.name
                    );
                    crate::metrics::record_trigger(
                        &trigger.name,
                        crate::metrics::TriggerOutcome::WhenNotMet,
                    );
                    return Ok(());
                }
                Err(e) => {
                    tracing::error!(
                        "[{event_id}] trigger '{}' `when` evaluation failed: {e}",
                        trigger.name
                    );
                    crate::metrics::record_trigger(
                        &trigger.name,
                        crate::metrics::TriggerOutcome::WhenError,
                    );
                    return Ok(());
                }
            }
        }

        if let Some(cooldown) = trigger.cooldown
            && !self.cooldown_ok(&trigger.name, cooldown).await?
        {
            tracing::info!(
                "[{event_id}] trigger '{}' within cooldown, skipping",
                trigger.name
            );
            crate::metrics::record_trigger(
                &trigger.name,
                crate::metrics::TriggerOutcome::CooldownSkipped,
            );
            return Ok(());
        }

        tracing::info!("[{event_id}] trigger '{}' fired", trigger.name);
        crate::metrics::record_trigger(&trigger.name, crate::metrics::TriggerOutcome::Fired);
        self.dispatch_workflow(event_id, trigger.run.clone())?;

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
        run: Vec<crate::settings::workflow::Step>,
    ) -> Result<(), ActorProcessingErr> {
        let Some(actor) = ractor::registry::where_is(WorkflowWorker::NAME.to_string()) else {
            tracing::warn!("[{event_id}] workflow factory not found, dropping trigger");
            return Ok(());
        };

        actor.send_message(FactoryMessage::Dispatch(Job {
            key: (),
            msg: WorkflowWorkerMessage::Execute {
                event_id,
                workflow: WorkflowSettings { enabled: true, run },
            },
            options: JobOptions::default(),
            accepted: None,
        }))?;

        Ok(())
    }
}

impl Actor for EventDispatcher {
    type Msg = EventBusMessage;
    type State = EventDispatcherState;
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

        Ok(EventDispatcherState::default())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = EventDispatcher::handle(self, message, state).await {
            tracing::error!("error while dispatching event: {e}");
        }

        Ok(())
    }
}
