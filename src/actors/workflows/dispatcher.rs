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
use crate::integrations::solar::SolarMetric;
use crate::lua::LuaAuthority;
use crate::repo::workflow::{NewPendingTimer, PendingTimerRow};
use crate::variables::Vars;
use crate::workflows::definition::Condition;
use crate::workflows::definition::{
    ReusableWorkflow, TriggerMatcher, Workflow, WorkflowDefinition,
};
use crate::workflows::manager::WorkflowManager;
use crate::workflows::timer_kind::TimerKind;
use crate::workflows::triggers::{
    EdgeLatches, PendingLatch, event_node, latch_for, solar_fires, unifi_matches, weather_fires,
};
use chrono::Utc;
use std::time::Duration;
use uuid::Uuid;

use ractor::{Actor, ActorProcessingErr, ActorRef};

use tracing::Instrument;

use crate::{
    actors::workflows::{WorkflowWorker, WorkflowWorkerMessage, conditions},
    event_bus::{
        BusEvent, EventBusMessage, EventSubscriber, Recipient, SensorMetric, Subscription,
    },
    integrations::solar::{queries, types::SolarCurrentStatisticsAverages},
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
    latches: EdgeLatches,
}

type EventSubject = (String, String);

fn schedule(myself: &ActorRef<DispatcherMessage>, timer: &PendingTimerRow) {
    let id = timer.id;
    let remaining = (timer.fire_at - Utc::now())
        .to_std()
        .unwrap_or(Duration::ZERO);

    myself.send_after(remaining, move || DispatcherMessage::TimerExpired(id));
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
                TriggerMatcher::GarageDoor { device, state },
                EventBusMessage::GarageDoor {
                    device_id,
                    state: s,
                    ..
                },
            ) => {
                devices.resolve_id(device) == Some(device_id.as_str())
                    && state.is_none_or(|state| state == *s)
            }
            (
                TriggerMatcher::AirPurifier { device, on, mode },
                EventBusMessage::AirPurifier {
                    device_id,
                    on: o,
                    mode: m,
                    ..
                },
            ) => {
                devices.resolve_id(device) == Some(device_id.as_str())
                    && on.is_none_or(|on| on == *o)
                    && mode.is_none_or(|mode| Some(mode) == *m)
            }
            (
                TriggerMatcher::Light { ieee_addr, on },
                EventBusMessage::Light {
                    ieee_addr: a,
                    on: o,
                    ..
                },
            ) => devices.address_or_self(ieee_addr) == a.as_str() && on.is_none_or(|on| on == *o),
            (
                TriggerMatcher::CommandFailed { device, kind },
                EventBusMessage::CommandFailed {
                    kind: k,
                    address,
                    device_id,
                    ..
                },
            ) => {
                kind.is_none_or(|kind| kind == *k)
                    && device.as_ref().is_none_or(|device| {
                        devices.address_or_self(device) == address.as_str()
                            || device_id.as_ref() == Some(device)
                    })
            }
            (
                TriggerMatcher::FeatureFlag { state },
                EventBusMessage::FeatureFlag { state: s, .. },
            ) => state.is_none_or(|state| state == *s),
            (
                TriggerMatcher::Switch { ieee_addr, action },
                EventBusMessage::SwitchAction {
                    ieee_addr: a,
                    action: ac,
                    ..
                },
            ) => {
                devices.address_or_self(ieee_addr) == a.as_str()
                    && action.as_ref().is_none_or(|action| action == ac)
            }
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
                    state.latches.last_satisfied.insert(key, false);
                    return false;
                }

                if state
                    .latches
                    .last_satisfied
                    .get(&key)
                    .copied()
                    .unwrap_or(false)
                {
                    return false;
                }

                *pending = Some(PendingLatch::Sensor(key));

                true
            }
            (
                TriggerMatcher::Plant { sensor, cmp },
                EventBusMessage::Plant {
                    sensor: s,
                    soil_moisture,
                    ..
                },
            ) => {
                if devices.address_or_self(sensor) != s.as_str() {
                    return false;
                }

                let satisfied = cmp.matches(*soil_moisture);
                let key = (workflow.name.clone(), s.clone(), SensorMetric::SoilMoisture);

                if !satisfied {
                    state.latches.last_satisfied.insert(key, false);
                    return false;
                }

                if state
                    .latches
                    .last_satisfied
                    .get(&key)
                    .copied()
                    .unwrap_or(false)
                {
                    return false;
                }

                *pending = Some(PendingLatch::Sensor(key));

                true
            }
            (TriggerMatcher::Cron { .. }, EventBusMessage::Cron { name, .. }) => {
                &workflow.name == name
            }
            (
                TriggerMatcher::Custom { name, source, .. },
                EventBusMessage::Custom {
                    name: n, source: s, ..
                },
            ) => name == n && source == s,
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
                TriggerMatcher::Unifi { clients, connected },
                EventBusMessage::Unifi {
                    mac_address,
                    client,
                    connected: c,
                    ..
                },
            ) => unifi_matches(clients.as_deref(), *connected, client, mac_address, *c),
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
                TriggerMatcher::DeviceConnection { device, connected },
                EventBusMessage::DeviceConnection {
                    device_id,
                    connected: is_connected,
                    ..
                },
            ) => {
                device.as_ref().is_none_or(|want| want == device_id)
                    && connected.is_none_or(|want| want == *is_connected)
            }
            (
                TriggerMatcher::Jellyfin {
                    state,
                    user,
                    devices,
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
                    && devices.as_ref().is_none_or(|devices| devices.contains(d))
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
                    &mut state.latches.last_solar_satisfied,
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
                        &mut state.latches.last_weather_satisfied,
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
        crate::telemetry::context::set_parent(&span, traceparent);

        match queries::statistics(&self.shared_actor_state.db)
            .instrument(span.clone())
            .await
        {
            Ok(statistics) => Some(statistics.averages),
            Err(e) => {
                tracing::error!("[{}] error reading solar averages: {e}", msg.event_id());
                crate::telemetry::context::record_error(&span, &e.to_string());
                None
            }
        }
    }

    async fn cancel_delayed(
        &self,
        msg: &EventBusMessage,
        settings: &crate::settings::Settings,
        subject: &EventSubject,
        traceparent: Option<&str>,
    ) {
        let event_id = msg.event_id();

        let delayed = settings
            .workflows
            .values()
            .filter_map(WorkflowDefinition::triggered)
            .any(|workflow| workflow.delay.is_some() && workflow.on.event_kind() == msg.kind());

        if !delayed {
            tracing::trace!(
                "[{event_id}] no delayed workflow listens for {}",
                msg.kind()
            );
            return;
        }

        let span = tracing::info_span!(
            parent: None,
            "dispatch.cancel_delayed",
            event_id = %event_id,
            event_kind = msg.kind(),
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        );
        crate::telemetry::context::set_parent(&span, traceparent);

        match self
            .shared_actor_state
            .repos
            .workflow()
            .cancel_timers_for_subject(TimerKind::Delay, &subject.0, &subject.1)
            .instrument(span.clone())
            .await
        {
            Ok(cancelled) => {
                for name in cancelled {
                    tracing::info!("[{event_id}] cancelled pending delayed trigger '{name}'");
                }
            }
            Err(e) => {
                tracing::error!("[{event_id}] failed to cancel delayed triggers: {e}");
                crate::telemetry::context::record_error(&span, &e.to_string());
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
        let averages = self
            .solar_averages(&msg, &settings, traceparent.as_deref())
            .await;

        let vars = Vars::default().with("event", event_node(&msg, averages.as_ref()));

        let subject: EventSubject = (msg.kind().to_string(), msg.entity());

        self.cancel_delayed(&msg, &settings, &subject, traceparent.as_deref())
            .await;

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
            let trigger_span = tracing::info_span!(
                parent: None,
                "trigger.evaluate",
                otel.name = format!("trigger.{}", workflow.slug),
                trigger = workflow.name,
                workflow = workflow.slug,
                event_kind = msg.kind(),
                event_id = %event_id,
            );
            crate::telemetry::context::set_parent(&trigger_span, traceparent.as_deref());

            let gated = async {
                self.shared_actor_state
                    .handles
                    .expect::<WorkflowManager>()
                    .enabled(&workflow.slug, workflow.enabled)
                    .await
                    && self.modes_active(event_id, workflow).await
            }
            .instrument(trigger_span.clone())
            .await;

            if !gated {
                continue;
            }

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
        vars: &Vars,
        state: &mut WorkflowDispatcherState,
        pending: Option<PendingLatch>,
    ) -> Result<(), ActorProcessingErr> {
        if !self.when_satisfied(event_id, workflow, vars).await {
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

        state.latches.commit(pending);

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
        vars: &Vars,
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

    async fn when_satisfied(&self, event_id: Uuid, workflow: &Workflow, vars: &Vars) -> bool {
        let Some(when) = &workflow.when else {
            return true;
        };

        match conditions::eval(&self.shared_actor_state, vars, when, &LuaAuthority::Trusted).await {
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
        vars: &Vars,
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
        let vars: Vars = match serde_json::from_value(timer.vars) {
            Ok(vars) => vars,
            Err(e) => {
                tracing::warn!(
                    "[{event_id}] dropping {} timer for '{}' with unreadable vars: {e}",
                    timer.timer_kind,
                    timer.workflow
                );
                return Ok(());
            }
        };
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
        vars: &Vars,
        state: &mut WorkflowDispatcherState,
    ) -> Result<(), ActorProcessingErr> {
        let Some(condition) = workflow.on.as_condition() else {
            tracing::warn!(
                "[{event_id}] trigger '{}' has a hold but no checkable state",
                workflow.name
            );
            return Ok(());
        };

        match conditions::eval(
            &self.shared_actor_state,
            vars,
            &Condition::Leaf(condition),
            &LuaAuthority::Trusted,
        )
        .await
        {
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

        if !self.when_satisfied(event_id, workflow, vars).await {
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

        state.latches.commit(latch_for(workflow, &subject.1));

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
        vars: Vars,
    ) -> Result<(), ActorProcessingErr> {
        Self::send_to_factory(event_id, workflow, vars)
    }

    fn send_to_factory(
        event_id: uuid::Uuid,
        workflow: ReusableWorkflow,
        vars: Vars,
    ) -> Result<(), ActorProcessingErr> {
        let message = WorkflowWorkerMessage::Execute {
            event_id,
            workflow,
            vars,
            authority: LuaAuthority::Trusted,
            traceparent: crate::telemetry::context::inject_current(),
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
