//! Event dispatcher: the single subscriber that turns bus events into workflow
//! runs.
//!
//! It subscribes to the in-memory [`EventBus`](crate::event_bus::EventBus),
//! matches each [`EventBusMessage`] against the configured `workflows:`, gates on
//! the optional `when` condition, and forwards matching workflows to the
//! `WorkflowWorker` factory. It deliberately does **no** workflow execution
//! itself — it only matches and dispatches, so the factory's worker pool keeps
//! providing the parallelism.

use crate::actors::system::rpc;
use crate::actors::workflows::manager::WorkflowManager;
use crate::repo::timer_kind::TimerKind;
use crate::repo::workflow::{NewPendingTimer, PendingTimerRow};
use crate::settings::workflow::Condition;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use ractor::{Actor, ActorProcessingErr, ActorRef};

use tracing::Instrument;

use crate::{
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage, conditions},
    event_bus::{
        BusEvent, EventBusMessage, EventSubscriber, ForecastDay, Recipient, SensorMetric,
        SolarMetric, Subscription, WeatherMetric, WeatherReading, WeatherSource,
    },
    integrations::solar::{queries, types::SolarCurrentStatisticsAverages},
    settings::{ReusableWorkflow, TriggerMatcher, Workflow, WorkflowDefinition},
    state::AppState,
};

pub struct WorkflowDispatcher {
    pub shared_actor_state: AppState,
}

pub struct DispatcherSubscriber;

pub enum DispatcherMessage {
    Event(Box<BusEvent>),
    TimerExpired(Uuid),
}

impl EventSubscriber for DispatcherSubscriber {
    type Msg = DispatcherMessage;

    const KINDS: &'static [&'static str] = EventBusMessage::KINDS;

    fn to_actor_message(&self, event: &BusEvent) -> Option<Self::Msg> {
        Some(DispatcherMessage::Event(Box::new(event.clone())))
    }
}

#[derive(Default)]
pub struct WorkflowDispatcherState {
    _subscription: Subscription,
    /// `(trigger name, sensor, metric) -> comparison satisfied at last reading`.
    /// Lets environment triggers fire on the rising edge only, matching the old
    /// plant-sensor semantics.
    last_satisfied: HashMap<(String, String, SensorMetric), bool>,
    /// `(trigger name, metric) -> comparison satisfied at last poll`, the solar
    /// counterpart to `last_satisfied` (there is only one plant, so no subject).
    last_solar_satisfied: HashMap<(String, SolarMetric), bool>,
    last_weather_satisfied: HashMap<WeatherKey, bool>,
}

type EventSubject = (String, String);

type WeatherKey = (String, WeatherSource, WeatherMetric, Option<ForecastDay>);

impl WorkflowDispatcherState {
    fn commit(&mut self, latch: Option<PendingLatch>) {
        match latch {
            Some(PendingLatch::Sensor(key)) => {
                self.last_satisfied.insert(key, true);
            }
            Some(PendingLatch::Solar(key)) => {
                self.last_solar_satisfied.insert(key, true);
            }
            Some(PendingLatch::Weather(key)) => {
                self.last_weather_satisfied.insert(key, true);
            }
            None => {}
        }
    }
}

fn latch_for(workflow: &Workflow, subject_entity: &str) -> Option<PendingLatch> {
    match &workflow.on {
        TriggerMatcher::Environment { metric, .. } => Some(PendingLatch::Sensor((
            workflow.name.clone(),
            subject_entity.to_owned(),
            metric.clone(),
        ))),
        TriggerMatcher::Solar { metric, .. } => {
            Some(PendingLatch::Solar((workflow.name.clone(), *metric)))
        }
        TriggerMatcher::Weather {
            source,
            metric,
            day,
            ..
        } => Some(PendingLatch::Weather((
            workflow.name.clone(),
            *source,
            *metric,
            *day,
        ))),
        _ => None,
    }
}

fn schedule(myself: &ActorRef<DispatcherMessage>, timer: &PendingTimerRow) {
    let id = timer.id;
    let remaining = (timer.fire_at - Utc::now())
        .to_std()
        .unwrap_or(Duration::ZERO);

    myself.send_after(remaining, move || DispatcherMessage::TimerExpired(id));
}

enum PendingLatch {
    Sensor((String, String, SensorMetric)),
    Solar((String, SolarMetric)),
    Weather(WeatherKey),
}

#[allow(clippy::too_many_arguments)]
fn weather_fires(
    name: &str,
    source: WeatherSource,
    metric: WeatherMetric,
    day: Option<ForecastDay>,
    cmp: &crate::settings::workflow::Comparison,
    readings: &[WeatherReading],
    last_satisfied: &mut HashMap<WeatherKey, bool>,
    pending: &mut Option<PendingLatch>,
) -> bool {
    let Some(reading) = readings
        .iter()
        .find(|reading| reading.metric == metric && reading.day == day)
    else {
        return false;
    };

    let satisfied = cmp.matches(reading.value);
    let key = (name.to_owned(), source, metric, day);

    if !satisfied {
        last_satisfied.insert(key, false);
        return false;
    }

    if last_satisfied.get(&key).copied().unwrap_or(false) {
        return false;
    }

    *pending = Some(PendingLatch::Weather(key));

    true
}

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
    pending: &mut Option<PendingLatch>,
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
    let key = (name.to_owned(), metric);

    if !satisfied {
        last_satisfied.insert(key, false);
        return false;
    }

    if last_satisfied.get(&key).copied().unwrap_or(false) {
        return false;
    }

    *pending = Some(PendingLatch::Solar(key));

    true
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
        pending: &mut Option<PendingLatch>,
    ) -> bool {
        let on = &workflow.on;
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

                if !satisfied {
                    state.last_satisfied.insert(key, false);
                    return false;
                }

                if state.last_satisfied.get(&key).copied().unwrap_or(false) {
                    return false;
                }

                *pending = Some(PendingLatch::Sensor(key));

                true
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
            (TriggerMatcher::Mode { to, from }, EventBusMessage::Mode { mode, previous, .. }) => {
                to.is_none_or(|to| to == *mode) && from.is_none_or(|from| from == *previous)
            }
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
                TriggerMatcher::FuelWatch {
                    change,
                    site_id,
                    below,
                    min_drop,
                },
                EventBusMessage::FuelWatch {
                    change: c,
                    site_id: id,
                    old_price,
                    new_price,
                    ..
                },
            ) => {
                change == c
                    && site_id.is_none_or(|s| s == *id)
                    && below.is_none_or(|b| *new_price < b)
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
            (TriggerMatcher::Solar { metric, cmp }, EventBusMessage::Solar { current_wh, .. }) => {
                solar_fires(
                    &workflow.name,
                    *metric,
                    cmp,
                    *current_wh,
                    averages,
                    &mut state.last_solar_satisfied,
                    pending,
                )
            }
            (
                TriggerMatcher::Weather {
                    source,
                    metric,
                    day,
                    cmp,
                },
                EventBusMessage::Weather {
                    source: s,
                    readings,
                    ..
                },
            ) => {
                source == s
                    && weather_fires(
                        &workflow.name,
                        *source,
                        *metric,
                        *day,
                        cmp,
                        readings,
                        &mut state.last_weather_satisfied,
                        pending,
                    )
            }
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
        traceparent: Option<&str>,
    ) -> Option<SolarCurrentStatisticsAverages> {
        if !matches!(msg, EventBusMessage::Solar { .. }) {
            return None;
        }

        let wanted = settings
            .workflows
            .values()
            .filter_map(WorkflowDefinition::triggered)
            .any(|workflow| {
                matches!(
                    &workflow.on,
                    TriggerMatcher::Solar { metric, .. } if *metric != SolarMetric::Current
                )
            });

        if !wanted {
            return None;
        }

        let span = tracing::info_span!(
            parent: None,
            "dispatch.solar_averages",
            event_id = %msg.event_id(),
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        );
        crate::tracing_context::set_parent(&span, traceparent);

        match queries::statistics(&self.shared_actor_state.db)
            .instrument(span.clone())
            .await
        {
            Ok(statistics) => Some(statistics.averages),
            Err(e) => {
                tracing::error!("[{}] error reading solar averages: {e}", msg.event_id());
                crate::tracing_context::record_error(&span, &e.to_string());
                None
            }
        }
    }

    async fn handle_event(
        &self,
        myself: &ActorRef<DispatcherMessage>,
        event: BusEvent,
        state: &mut WorkflowDispatcherState,
    ) -> Result<(), ActorProcessingErr> {
        let BusEvent {
            traceparent,
            message: msg,
        } = event;

        let event_id = msg.event_id();
        crate::metrics::record_event(msg.kind());
        let settings = self.shared_actor_state.settings.clone();
        let mut vars = msg.vars();

        let averages = self
            .solar_averages(&msg, &settings, traceparent.as_deref())
            .await;
        if let Some(averages) = &averages {
            let watts = |w: Option<f64>| w.map_or_else(String::new, |w| format!("{w:.0}"));
            vars.insert("avg_15m".to_owned(), watts(averages.last_15_mins));
            vars.insert("avg_1h".to_owned(), watts(averages.last_1_hour));
            vars.insert("avg_3h".to_owned(), watts(averages.last_3_hours));
        }

        let subject: EventSubject = (msg.kind().to_string(), msg.entity());

        match self
            .shared_actor_state
            .repos
            .workflow()
            .cancel_timers_for_subject(TimerKind::Delay, &subject.0, &subject.1)
            .await
        {
            Ok(cancelled) => {
                for name in cancelled {
                    tracing::info!("[{event_id}] cancelled pending delayed trigger '{name}'");
                }
            }
            Err(e) => tracing::error!("[{event_id}] failed to cancel delayed triggers: {e}"),
        }

        for workflow in settings
            .workflows
            .values()
            .filter_map(WorkflowDefinition::triggered)
        {
            let mut pending = None;

            if !self.matches(workflow, &msg, averages.as_ref(), state, &mut pending) {
                if workflow.hold.is_some() && workflow.on.event_kind() == msg.kind() {
                    self.cancel_hold(event_id, workflow, &subject).await;
                }

                continue;
            }
            if !self
                .shared_actor_state
                .handles
                .expect::<WorkflowManager>()
                .enabled(&workflow.slug, workflow.enabled)
                .await
            {
                continue;
            }

            if !self.modes_active(event_id, workflow).await {
                continue;
            }

            let trigger_span = tracing::info_span!(
                parent: None,
                "trigger.evaluate",
                otel.name = format!("trigger: {}", workflow.name),
                trigger = workflow.name,
                event_kind = msg.kind(),
                event_id = %event_id,
            );
            crate::tracing_context::set_parent(&trigger_span, traceparent.as_deref());

            self.evaluate_trigger(myself, event_id, workflow, &subject, &vars, state, pending)
                .instrument(trigger_span)
                .await?;
        }

        Ok(())
    }

    /// Evaluate a single matched trigger: gate on `when`, honour the cooldown,
    /// and dispatch its workflow. Recorded as one `trigger.evaluate` span by the
    /// caller via [`Instrument`].
    #[allow(clippy::too_many_arguments)]
    async fn evaluate_trigger(
        &self,
        myself: &ActorRef<DispatcherMessage>,
        event_id: uuid::Uuid,
        workflow: &Workflow,
        subject: &EventSubject,
        vars: &HashMap<String, String>,
        state: &mut WorkflowDispatcherState,
        pending: Option<PendingLatch>,
    ) -> Result<(), ActorProcessingErr> {
        if !self.when_satisfied(event_id, workflow).await {
            return Ok(());
        }

        if let Some(hold) = workflow.hold {
            self.arm_timer(
                myself,
                event_id,
                workflow,
                TimerKind::Hold,
                hold,
                subject,
                vars,
            )
            .await;

            return Ok(());
        }

        if let Some(cooldown) = workflow.cooldown
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

        state.commit(pending);

        tracing::info!("[{event_id}] trigger '{}' fired", workflow.name);
        crate::metrics::record_trigger(&workflow.name, crate::metrics::TriggerOutcome::Fired);

        self.dispatch_or_delay(myself, event_id, workflow, subject, vars)
            .await
    }

    async fn dispatch_or_delay(
        &self,
        myself: &ActorRef<DispatcherMessage>,
        event_id: Uuid,
        workflow: &Workflow,
        subject: &EventSubject,
        vars: &HashMap<String, String>,
    ) -> Result<(), ActorProcessingErr> {
        match workflow.delay {
            Some(delay) => {
                self.arm_timer(
                    myself,
                    event_id,
                    workflow,
                    TimerKind::Delay,
                    delay,
                    subject,
                    vars,
                )
                .await;
            }
            None => self.dispatch_workflow(event_id, workflow.body.clone(), vars.clone())?,
        }

        Ok(())
    }

    async fn modes_active(&self, event_id: Uuid, workflow: &Workflow) -> bool {
        let active = self
            .shared_actor_state
            .handles
            .expect::<WorkflowManager>()
            .any_mode_active(&workflow.modes)
            .await;

        if !active {
            tracing::info!(
                "[{event_id}] trigger '{}' matched but none of its modes are active",
                workflow.name
            );
        }

        active
    }

    async fn when_satisfied(&self, event_id: Uuid, workflow: &Workflow) -> bool {
        let Some(when) = &workflow.when else {
            return true;
        };

        match conditions::eval(&self.shared_actor_state, when).await {
            Ok(true) => true,
            Ok(false) => {
                tracing::info!(
                    "[{event_id}] trigger '{}' matched but `when` not satisfied",
                    workflow.name
                );
                crate::metrics::record_trigger(
                    &workflow.name,
                    crate::metrics::TriggerOutcome::WhenNotMet,
                );
                false
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
                false
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn arm_timer(
        &self,
        myself: &ActorRef<DispatcherMessage>,
        event_id: Uuid,
        workflow: &Workflow,
        kind: TimerKind,
        duration: chrono::TimeDelta,
        subject: &EventSubject,
        vars: &HashMap<String, String>,
    ) {
        let vars = match serde_json::to_value(vars) {
            Ok(vars) => vars,
            Err(e) => {
                tracing::error!(
                    "[{event_id}] failed to encode vars for '{}': {e}",
                    workflow.name
                );
                return;
            }
        };

        let armed = self
            .shared_actor_state
            .repos
            .workflow()
            .arm_timer(NewPendingTimer {
                workflow: &workflow.name,
                kind,
                subject_kind: &subject.0,
                subject_entity: &subject.1,
                event_id,
                vars,
                fire_at: Utc::now() + duration,
            })
            .await;

        match armed {
            Ok(Some(timer)) => {
                tracing::info!(
                    "[{event_id}] trigger '{}' armed {} timer for {}",
                    workflow.name,
                    kind.as_str(),
                    crate::timedelta_format::humanize(duration)
                );

                schedule(myself, &timer);
            }
            Ok(None) => {
                tracing::info!(
                    "[{event_id}] trigger '{}' is already holding, keeping its deadline",
                    workflow.name
                );
            }
            Err(e) => {
                tracing::error!(
                    "[{event_id}] failed to arm {} timer for '{}': {e}",
                    kind.as_str(),
                    workflow.name
                );
            }
        }
    }

    async fn cancel_hold(&self, event_id: Uuid, workflow: &Workflow, subject: &EventSubject) {
        let cancelled = self
            .shared_actor_state
            .repos
            .workflow()
            .cancel_timer(&workflow.name, TimerKind::Hold, &subject.0, &subject.1)
            .await;

        match cancelled {
            Ok(true) => {
                tracing::info!(
                    "[{event_id}] trigger '{}' no longer holds, cancelled its timer",
                    workflow.name
                );
            }
            Ok(false) => {}
            Err(e) => {
                tracing::error!(
                    "[{event_id}] failed to cancel hold for '{}': {e}",
                    workflow.name
                );
            }
        }
    }

    async fn handle_timer(
        &self,
        myself: &ActorRef<DispatcherMessage>,
        id: Uuid,
        state: &mut WorkflowDispatcherState,
    ) -> Result<(), ActorProcessingErr> {
        let Some(timer) = self
            .shared_actor_state
            .repos
            .workflow()
            .take_timer(id)
            .await?
        else {
            tracing::debug!("timer {id} was cancelled before it fired");
            return Ok(());
        };

        let settings = self.shared_actor_state.settings.clone();

        let Some(workflow) = settings
            .workflows
            .get(&timer.workflow)
            .and_then(WorkflowDefinition::triggered)
        else {
            tracing::warn!(
                "dropping {} timer for unknown workflow '{}'",
                timer.timer_kind,
                timer.workflow
            );
            return Ok(());
        };

        let event_id = timer.event_id;
        let vars: HashMap<String, String> = serde_json::from_value(timer.vars)?;
        let subject: EventSubject = (timer.subject_kind, timer.subject_entity);

        match TimerKind::parse(&timer.timer_kind) {
            Some(TimerKind::Hold) => {
                self.fire_hold(myself, event_id, workflow, &subject, &vars, state)
                    .await
            }
            Some(TimerKind::Delay) => {
                if !self.modes_active(event_id, workflow).await {
                    return Ok(());
                }

                tracing::info!("[{event_id}] delayed trigger '{}' firing", workflow.name);
                self.dispatch_workflow(event_id, workflow.body.clone(), vars)
            }
            None => {
                tracing::warn!(
                    "[{event_id}] dropping timer with unknown kind '{}'",
                    timer.timer_kind
                );
                Ok(())
            }
        }
    }

    async fn fire_hold(
        &self,
        myself: &ActorRef<DispatcherMessage>,
        event_id: Uuid,
        workflow: &Workflow,
        subject: &EventSubject,
        vars: &HashMap<String, String>,
        state: &mut WorkflowDispatcherState,
    ) -> Result<(), ActorProcessingErr> {
        let Some(condition) = workflow.on.as_condition() else {
            tracing::warn!(
                "[{event_id}] trigger '{}' has a hold but no checkable state",
                workflow.name
            );
            return Ok(());
        };

        match conditions::eval(&self.shared_actor_state, &Condition::Leaf(condition)).await {
            Ok(true) => {}
            Ok(false) => {
                tracing::info!(
                    "[{event_id}] trigger '{}' did not hold for its `for:` duration",
                    workflow.name
                );
                return Ok(());
            }
            Err(e) => {
                tracing::error!(
                    "[{event_id}] trigger '{}' hold re-check failed: {e}",
                    workflow.name
                );
                return Ok(());
            }
        }

        if !self.when_satisfied(event_id, workflow).await {
            return Ok(());
        }

        if !self.modes_active(event_id, workflow).await {
            return Ok(());
        }

        if let Some(cooldown) = workflow.cooldown
            && !self.cooldown_ok(&workflow.name, cooldown).await?
        {
            tracing::info!(
                "[{event_id}] trigger '{}' held but is within cooldown, skipping",
                workflow.name
            );
            crate::metrics::record_trigger(
                &workflow.name,
                crate::metrics::TriggerOutcome::CooldownSkipped,
            );
            return Ok(());
        }

        state.commit(latch_for(workflow, &subject.1));

        tracing::info!("[{event_id}] trigger '{}' held and fired", workflow.name);
        crate::metrics::record_trigger(&workflow.name, crate::metrics::TriggerOutcome::Fired);

        self.dispatch_or_delay(myself, event_id, workflow, subject, vars)
            .await
    }

    async fn restore_timers(&self, myself: &ActorRef<DispatcherMessage>) {
        let repo = self.shared_actor_state.repos.workflow();

        let timers = match repo.pending_timers().await {
            Ok(timers) => timers,
            Err(e) => {
                tracing::error!("failed to load pending workflow timers: {e}");
                return;
            }
        };

        let catch_up_within = self
            .shared_actor_state
            .settings
            .workflow
            .timers
            .catch_up_within;
        let now = Utc::now();

        for timer in timers {
            if now - timer.fire_at > catch_up_within {
                tracing::warn!(
                    "dropping {} timer for '{}' that was due at {}",
                    timer.timer_kind,
                    timer.workflow,
                    timer.fire_at
                );

                if let Err(e) = repo.take_timer(timer.id).await {
                    tracing::error!("failed to drop stale timer {}: {e}", timer.id);
                }

                continue;
            }

            tracing::info!(
                "restoring {} timer for '{}' due at {}",
                timer.timer_kind,
                timer.workflow,
                timer.fire_at
            );

            schedule(myself, &timer);
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
        let ok = self
            .shared_actor_state
            .handles
            .expect::<WorkflowManager>()
            .cooldown_ok(name, cooldown)
            .await?;

        Ok(ok)
    }

    fn dispatch_workflow(
        &self,
        event_id: uuid::Uuid,
        workflow: ReusableWorkflow,
        vars: HashMap<String, String>,
    ) -> Result<(), ActorProcessingErr> {
        Self::send_to_factory(event_id, workflow, vars)
    }

    fn send_to_factory(
        event_id: uuid::Uuid,
        workflow: ReusableWorkflow,
        vars: HashMap<String, String>,
    ) -> Result<(), ActorProcessingErr> {
        let message = WorkflowWorkerMessage::Execute {
            event_id,
            workflow,
            vars,
            traceparent: crate::tracing_context::inject_current(),
        };

        if let Err(e) = rpc::cast_factory(WorkflowWorker::NAME, message) {
            tracing::warn!("[{event_id}] could not dispatch trigger: {e}");
        }

        Ok(())
    }
}

impl Actor for WorkflowDispatcher {
    type Msg = DispatcherMessage;
    type State = WorkflowDispatcherState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        // the bus forwards every event straight into this mailbox, so matching is
        // serialized through `handle` while execution fans out to the factory
        let subscription = self.shared_actor_state.event_bus.register(
            Self::NAME,
            Recipient::Actor(myself.clone()),
            DispatcherSubscriber,
        );

        self.restore_timers(&myself).await;

        Ok(WorkflowDispatcherState {
            _subscription: subscription,
            ..Default::default()
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let result = match message {
            DispatcherMessage::Event(event) => self.handle_event(&myself, *event, state).await,
            DispatcherMessage::TimerExpired(id) => self.handle_timer(&myself, id, state).await,
        };

        if let Err(e) = result {
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
            let mut pending = None;
            let fired = solar_fires(
                "solar surplus",
                SolarMetric::Avg15m,
                &cmp,
                0.0,
                Some(&averages(Some(avg))),
                last,
                &mut pending,
            );

            if let Some(PendingLatch::Solar(key)) = pending {
                last.insert(key, true);
            }

            fired
        };

        assert!(fires(3500.0, &mut last), "expected the crossing to fire");
        assert!(!fires(4000.0, &mut last), "expected no re-fire while past");
        assert!(!fires(2000.0, &mut last), "expected no fire on the drop");
        assert!(fires(3500.0, &mut last), "expected a re-arm and re-fire");
    }

    fn workflow(yaml: &str) -> Workflow {
        serde_yaml::from_str(yaml).expect("workflow yaml")
    }

    #[test]
    fn a_held_threshold_latches_the_edge_it_fired_on() {
        let hot = workflow(
            r#"
name: hot
on: { type: weather, source: bom, metric: temperature, op: gt, value: 35 }
for: 1h
modes: [home]
run: []
"#,
        );

        let mut state = WorkflowDispatcherState::default();
        state.commit(latch_for(&hot, "bom"));

        assert_eq!(
            state.last_weather_satisfied.get(&(
                "hot".to_owned(),
                WeatherSource::Bom,
                WeatherMetric::Temperature,
                None
            )),
            Some(&true)
        );
    }

    #[test]
    fn a_held_environment_trigger_latches_on_the_event_sensor() {
        let damp = workflow(
            r#"
name: damp
on: { type: environment, sensor: bathroom, metric: humidity, op: gt, value: 80 }
for: 10m
modes: [home]
run: []
"#,
        );

        assert!(matches!(
            latch_for(&damp, "0xabc"),
            Some(PendingLatch::Sensor((name, sensor, SensorMetric::Humidity)))
                if name == "damp" && sensor == "0xabc"
        ));
    }

    #[test]
    fn presence_holds_have_no_edge_to_latch() {
        let empty = workflow(
            r#"
name: empty
on: { type: presence, sensor: hallway, present: false }
for: 30m
modes: [home]
run: []
"#,
        );

        assert!(latch_for(&empty, "0x1").is_none());
        assert!(empty.on.supports_hold());
    }

    #[test]
    fn a_crossing_rejected_by_a_guard_still_fires_when_the_guard_opens() {
        let cmp = Comparison {
            op: CompareOp::Gt,
            value: 3000.0,
        };
        let mut last = HashMap::new();

        let evaluate = |avg: f64, last: &mut HashMap<_, _>, guard_open: bool| {
            let mut pending = None;
            let fired = solar_fires(
                "solar surplus",
                SolarMetric::Avg15m,
                &cmp,
                0.0,
                Some(&averages(Some(avg))),
                last,
                &mut pending,
            );

            if fired
                && guard_open
                && let Some(PendingLatch::Solar(key)) = pending
            {
                last.insert(key, true);
            }

            fired && guard_open
        };

        assert!(
            !evaluate(3500.0, &mut last, false),
            "the guard is shut, so nothing fires"
        );
        assert!(
            evaluate(3500.0, &mut last, true),
            "the edge was not consumed by the shut guard, so it fires now"
        );
        assert!(
            !evaluate(3500.0, &mut last, true),
            "the edge is consumed once it has fired"
        );
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
            &mut None,
        ));
        assert!(!solar_fires(
            "low solar",
            SolarMetric::Avg1h,
            &cmp,
            0.0,
            None,
            &mut last,
            &mut None,
        ));
        assert!(last.is_empty(), "edge state should be untouched");
    }

    #[test]
    fn a_weather_forecast_fires_on_the_rising_edge_for_its_day_only() {
        let cmp = Comparison {
            op: CompareOp::Gte,
            value: 35.0,
        };
        let mut last = HashMap::new();

        let forecast = |today: f64, tomorrow: f64| {
            vec![
                WeatherReading {
                    metric: WeatherMetric::MaxTemp,
                    day: Some(ForecastDay::Today),
                    value: today,
                },
                WeatherReading {
                    metric: WeatherMetric::MaxTemp,
                    day: Some(ForecastDay::Tomorrow),
                    value: tomorrow,
                },
            ]
        };

        let fires = |readings: Vec<WeatherReading>, last: &mut HashMap<_, _>| {
            let mut pending = None;
            let fired = weather_fires(
                "hot tomorrow",
                WeatherSource::WillyWeather,
                WeatherMetric::MaxTemp,
                Some(ForecastDay::Tomorrow),
                &cmp,
                &readings,
                last,
                &mut pending,
            );

            if let Some(PendingLatch::Weather(key)) = pending {
                last.insert(key, true);
            }

            fired
        };

        assert!(!fires(forecast(38.0, 30.0), &mut last), "today is ignored");
        assert!(fires(forecast(20.0, 36.0), &mut last), "tomorrow crosses");
        assert!(
            !fires(forecast(20.0, 37.0), &mut last),
            "no re-fire while past"
        );
        assert!(!fires(forecast(20.0, 30.0), &mut last), "drops back");
        assert!(fires(forecast(20.0, 35.0), &mut last), "re-arms");
    }

    #[test]
    fn a_rain_probability_forecast_fires_on_the_rising_edge() {
        let cmp = Comparison {
            op: CompareOp::Gte,
            value: 70.0,
        };
        let mut last = HashMap::new();

        let fires = |probability: f64, last: &mut HashMap<_, _>| {
            let readings = vec![WeatherReading {
                metric: WeatherMetric::RainProbability,
                day: Some(ForecastDay::Tomorrow),
                value: probability,
            }];

            let mut pending = None;
            let fired = weather_fires(
                "rain tomorrow",
                WeatherSource::WillyWeather,
                WeatherMetric::RainProbability,
                Some(ForecastDay::Tomorrow),
                &cmp,
                &readings,
                last,
                &mut pending,
            );

            if let Some(PendingLatch::Weather(key)) = pending {
                last.insert(key, true);
            }

            fired
        };

        assert!(!fires(40.0, &mut last), "below the threshold");
        assert!(fires(80.0, &mut last), "crosses the threshold");
        assert!(!fires(90.0, &mut last), "no re-fire while past");
        assert!(!fires(30.0, &mut last), "drops back");
        assert!(fires(75.0, &mut last), "re-arms");
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
            &mut None,
        ));
    }
}
