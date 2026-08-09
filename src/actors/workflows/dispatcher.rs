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
    event_bus::{EventBusMessage, SensorMetric, SolarMetric},
    integrations::solar::{queries, types::SolarCurrentStatisticsAverages},
    settings::{TriggerMatcher, Workflow},
    state::SharedActorState,
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
    /// `(trigger name, metric) -> comparison satisfied at last poll`, the solar
    /// counterpart to `last_satisfied` (there is only one plant, so no subject).
    last_solar_satisfied: HashMap<(String, SolarMetric), bool>,
    pending_delays: HashMap<(EventSubject, String), tokio::task::JoinHandle<()>>,
}

type EventSubject = (String, String);

/// Whether a solar trigger fires for this reading: pick the metric's value
/// (`None` when its average window is still empty, which never fires and leaves
/// the edge state untouched), compare it, and gate on the rising edge so a value
/// sitting past the threshold only fires once.
fn solar_fires(
    name: &str,
    metric: SolarMetric,
    cmp: &crate::settings::workflow::Comparison,
    current_wh: f64,
    averages: Option<&SolarCurrentStatisticsAverages>,
    last_satisfied: &mut HashMap<(String, SolarMetric), bool>,
) -> bool {
    let value = match metric {
        SolarMetric::Current => Some(current_wh),
        SolarMetric::Avg15m => averages.and_then(|a| a.last_15_mins),
        SolarMetric::Avg1h => averages.and_then(|a| a.last_1_hour),
        SolarMetric::Avg3h => averages.and_then(|a| a.last_3_hours),
    };

    let Some(value) = value else {
        return false;
    };

    let satisfied = cmp.matches(value);
    let was_satisfied = last_satisfied
        .insert((name.to_owned(), metric), satisfied)
        .unwrap_or(false);

    satisfied && !was_satisfied
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
        averages: Option<&SolarCurrentStatisticsAverages>,
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
                    sensor: s,
                    readings,
                    ..
                },
            ) => {
                if devices.address_or_self(sensor) != s.as_str() {
                    return false;
                }
                let Some(reading) = readings.iter().find(|r| r.metric() == *metric) else {
                    return false;
                };
                let satisfied = cmp.matches(reading.value());
                let key = (workflow.name.clone(), s.clone(), reading.metric());
                let was_satisfied = state.last_satisfied.insert(key, satisfied).unwrap_or(false);
                // rising edge only: fire when the threshold is newly crossed
                satisfied && !was_satisfied
            }
            (TriggerMatcher::Cron { .. }, EventBusMessage::Cron { name, .. }) => {
                &workflow.name == name
            }
            (
                TriggerMatcher::Sun { transition, offset },
                EventBusMessage::Sun {
                    transition: t,
                    offset: o,
                    ..
                },
            ) => transition == t && offset == o,
            (
                TriggerMatcher::Mode { mode, active },
                EventBusMessage::Mode {
                    mode: m, active: a, ..
                },
            ) => mode == m && active == a,
            (
                TriggerMatcher::HomeAssistant { entity_id, state },
                EventBusMessage::HomeAssistant {
                    entity_id: e,
                    state: s,
                    ..
                },
            ) => entity_id == e && state.as_ref().is_none_or(|state| state == s),
            (
                TriggerMatcher::Woolworths {
                    product_id,
                    min_drop,
                },
                EventBusMessage::Woolworths {
                    product_id: id,
                    old_price,
                    new_price,
                    ..
                },
            ) => {
                product_id.is_none_or(|p| p == *id)
                    && min_drop.is_none_or(|min| old_price - new_price >= min)
            }
            (
                TriggerMatcher::DeviceBattery {
                    device_id,
                    kind,
                    below,
                },
                EventBusMessage::DeviceBattery {
                    device_id: id,
                    kind: k,
                    battery_voltage,
                    battery_percent,
                    ..
                },
            ) => {
                let level = battery_percent.or(*battery_voltage);
                device_id.as_ref().is_none_or(|d| d == id)
                    && kind.as_ref().is_none_or(|want| want == k)
                    && below.is_none_or(|threshold| level.is_some_and(|l| l < threshold))
            }
            (
                TriggerMatcher::Jellyfin {
                    state,
                    user,
                    device,
                    item_type,
                },
                EventBusMessage::Jellyfin {
                    state: s,
                    user: u,
                    device: d,
                    item_type: t,
                    ..
                },
            ) => {
                state.is_none_or(|state| state == *s)
                    && user.as_ref().is_none_or(|user| user == u)
                    && device.as_ref().is_none_or(|device| device == d)
                    && item_type.as_ref().is_none_or(|item_type| item_type == t)
            }
            (
                TriggerMatcher::MediaPlayer { device, state, app },
                EventBusMessage::MediaPlayer {
                    device_id: d,
                    state: s,
                    app_name: a,
                    ..
                },
            ) => {
                state.is_none_or(|state| state == *s)
                    && device.as_ref().is_none_or(|device| device == d)
                    && app
                        .as_ref()
                        .is_none_or(|app| a.as_ref().is_some_and(|actual| actual == app))
            }
            (
                TriggerMatcher::Solar { metric, cmp },
                EventBusMessage::Solar { current_wh, .. },
            ) => solar_fires(
                &workflow.name,
                *metric,
                cmp,
                *current_wh,
                averages,
                &mut state.last_solar_satisfied,
            ),
            _ => false,
        }
    }

    /// Rolling solar averages, read from the DB only when a solar event arrives
    /// and some configured trigger actually asks for an average — a `current`-only
    /// setup never pays for the query. A failure is logged and treated as "no
    /// averages", so an average trigger simply doesn't match this poll.
    async fn solar_averages(
        &self,
        msg: &EventBusMessage,
        settings: &crate::settings::Settings,
    ) -> Option<SolarCurrentStatisticsAverages> {
        if !matches!(msg, EventBusMessage::Solar { .. }) {
            return None;
        }

        let wanted = settings.workflows.values().any(|workflow| {
            matches!(
                workflow.on(),
                Some(TriggerMatcher::Solar { metric, .. }) if *metric != SolarMetric::Current
            )
        });

        if !wanted {
            return None;
        }

        match queries::statistics(&self.shared_actor_state.db).await {
            Ok(statistics) => Some(statistics.averages),
            Err(e) => {
                tracing::error!(
                    "[{}] error reading solar averages: {e}",
                    msg.event_id()
                );
                None
            }
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
        let mut vars = msg.vars();

        let averages = self.solar_averages(&msg, &settings).await;
        if let Some(averages) = &averages {
            let watts = |w: Option<f64>| w.map_or_else(String::new, |w| format!("{w:.0}"));
            vars.insert("avg_15m".to_owned(), watts(averages.last_15_mins));
            vars.insert("avg_1h".to_owned(), watts(averages.last_1_hour));
            vars.insert("avg_3h".to_owned(), watts(averages.last_3_hours));
        }

        let subject: EventSubject = (msg.kind().to_string(), msg.entity());
        state.pending_delays.retain(|(s, name), handle| {
            if s == &subject {
                handle.abort();
                tracing::info!("[{event_id}] cancelled pending delayed trigger '{name}'");
                false
            } else {
                true
            }
        });

        for workflow in settings.workflows.values() {
            if !self.matches(workflow, &msg, averages.as_ref(), state) {
                continue;
            }
            if !self
                .shared_actor_state
                .workflows
                .enabled(&workflow.slug, workflow.enabled)
                .await
            {
                continue;
            }

            let trigger_span = tracing::info_span!(
                "trigger.evaluate",
                otel.name = format!("trigger: {}", workflow.name),
                trigger = workflow.name,
                event_kind = msg.kind(),
            );
            self.evaluate_trigger(event_id, workflow, &subject, &vars, state)
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
        subject: &EventSubject,
        vars: &HashMap<String, String>,
        state: &mut WorkflowDispatcherState,
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

        match workflow.delay() {
            Some(delay) => self.schedule_delayed(event_id, workflow, delay, subject, vars, state),
            None => self.dispatch_workflow(event_id, workflow.clone(), vars.clone())?,
        }

        Ok(())
    }

    fn schedule_delayed(
        &self,
        event_id: uuid::Uuid,
        workflow: &Workflow,
        delay: chrono::TimeDelta,
        subject: &EventSubject,
        vars: &HashMap<String, String>,
        state: &mut WorkflowDispatcherState,
    ) {
        let Ok(delay) = delay.to_std() else {
            let _ = self.dispatch_workflow(event_id, workflow.clone(), vars.clone());
            return;
        };

        let name = workflow.name.clone();
        let workflow = workflow.clone();
        tracing::info!(
            "[{event_id}] trigger '{name}' deferred by {}s",
            delay.as_secs()
        );

        let task_name = name.clone();
        let vars = vars.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            tracing::info!("[{event_id}] delayed trigger '{task_name}' firing");
            if let Err(e) = Self::send_to_factory(event_id, workflow, vars) {
                tracing::error!(
                    "[{event_id}] failed to dispatch delayed trigger '{task_name}': {e}"
                );
            }
        });

        if let Some(prev) = state.pending_delays.insert((subject.clone(), name), handle) {
            prev.abort();
        }
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
        vars: HashMap<String, String>,
    ) -> Result<(), ActorProcessingErr> {
        Self::send_to_factory(event_id, workflow, vars)
    }

    fn send_to_factory(
        event_id: uuid::Uuid,
        workflow: Workflow,
        vars: HashMap<String, String>,
    ) -> Result<(), ActorProcessingErr> {
        let Some(actor) = ractor::registry::where_is(WorkflowWorker::NAME.to_string()) else {
            tracing::warn!("[{event_id}] workflow factory not found, dropping trigger");
            return Ok(());
        };

        actor.send_message(FactoryMessage::Dispatch(Job {
            key: (),
            msg: WorkflowWorkerMessage::Execute {
                event_id,
                workflow,
                vars,
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::workflow::{CompareOp, Comparison};

    fn averages(last_15_mins: Option<f64>) -> SolarCurrentStatisticsAverages {
        SolarCurrentStatisticsAverages {
            last_15_mins,
            last_1_hour: None,
            last_3_hours: None,
        }
    }

    #[test]
    fn a_sustained_average_fires_once_and_rearms_after_dropping_back() {
        let cmp = Comparison {
            op: CompareOp::Gt,
            value: 3000.0,
        };
        let mut last = HashMap::new();

        let fires = |avg: f64, last: &mut HashMap<_, _>| {
            solar_fires(
                "solar surplus",
                SolarMetric::Avg15m,
                &cmp,
                0.0,
                Some(&averages(Some(avg))),
                last,
            )
        };

        assert!(fires(3500.0, &mut last), "expected the crossing to fire");
        assert!(!fires(4000.0, &mut last), "expected no re-fire while past");
        assert!(!fires(2000.0, &mut last), "expected no fire on the drop");
        assert!(fires(3500.0, &mut last), "expected a re-arm and re-fire");
    }

    #[test]
    fn an_empty_average_window_never_fires() {
        let cmp = Comparison {
            op: CompareOp::Lt,
            value: 500.0,
        };
        let mut last = HashMap::new();

        assert!(!solar_fires(
            "low solar",
            SolarMetric::Avg1h,
            &cmp,
            0.0,
            Some(&averages(None)),
            &mut last,
        ));
        assert!(!solar_fires(
            "low solar",
            SolarMetric::Avg1h,
            &cmp,
            0.0,
            None,
            &mut last,
        ));
        assert!(last.is_empty(), "edge state should be untouched");
    }

    #[test]
    fn the_current_metric_reads_the_event_not_the_averages() {
        let cmp = Comparison {
            op: CompareOp::Gt,
            value: 1000.0,
        };
        let mut last = HashMap::new();

        assert!(solar_fires(
            "solar on",
            SolarMetric::Current,
            &cmp,
            1500.0,
            None,
            &mut last,
        ));
    }
}
