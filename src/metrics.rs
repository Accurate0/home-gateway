//! Domain metrics for trigger dispatch and workflow execution.
//!
//! Mirrors the janitor-bot pattern: a small module of `record_*` helpers that
//! the dispatcher and workflow executor call, keeping instrument names and
//! label conventions in one place. Instruments are created lazily against the
//! global meter set up in [`crate::tracing_setup::init_metrics`], so they're
//! exported via the same Prometheus `/metrics` endpoint.

use std::sync::LazyLock;
use std::time::Duration;

use opentelemetry::metrics::{Counter, Gauge, Histogram};
use opentelemetry::{KeyValue, global};

struct Instruments {
    /// Bus events seen by the dispatcher, labelled by event kind.
    events_total: Counter<u64>,
    /// Trigger firings, labelled by trigger name and outcome.
    triggers_total: Counter<u64>,
    /// Workflow executions, labelled by outcome (`success` / `error` / `disabled`).
    workflows_total: Counter<u64>,
    /// Wall-clock time to execute a whole workflow.
    workflow_duration: Histogram<f64>,
    /// Individual workflow steps, labelled by step kind and outcome.
    steps_total: Counter<u64>,
    /// Wall-clock time to execute a single step.
    step_duration: Histogram<f64>,
    /// Latest reported device battery voltage, labelled by device id and kind.
    device_battery_voltage: Gauge<f64>,
    device_battery_percent: Gauge<f64>,
    /// Signed seconds between a display's intended wake and its actual poll.
    eink_wake_drift: Histogram<f64>,
    /// Ad-hoc task outcomes, labelled by task name and outcome.
    adhoc_tasks_total: Counter<u64>,
    /// Integration poll outcomes, labelled by integration name and outcome.
    integration_polls_total: Counter<u64>,
    /// Wall-clock time for a single integration poll.
    integration_poll_duration: Histogram<f64>,
    /// MQTT messages ingested, labelled by classified topic kind.
    mqtt_ingest_total: Counter<u64>,
    /// Supervised actor restarts, labelled by actor name and reason.
    actor_restarts_total: Counter<u64>,
    /// Device handler message failures, labelled by handler name.
    device_handler_errors_total: Counter<u64>,
    /// Seconds since a watchdog device last reported, labelled by device key.
    device_last_seen_age: Gauge<f64>,
    /// Whether a watchdog device is past its timeout (1) or reporting (0).
    device_stale: Gauge<u64>,
    /// Reconciler command re-drives, labelled by device kind.
    reconciler_retries_total: Counter<u64>,
    /// Reconciler commands abandoned after `max_attempts`, by kind and device.
    reconciler_give_ups_total: Counter<u64>,
    lua_duration: Histogram<f64>,
    lua_memory: Histogram<u64>,
    lua_vm_setup_duration: Histogram<f64>,
    rpc_errors_total: Counter<u64>,
    db_queries_total: Counter<u64>,
    db_query_duration: Histogram<f64>,
    graphql_operations_total: Counter<u64>,
    graphql_operation_duration: Histogram<f64>,
    /// REST requests, labelled by method, matched route and status code.
    rest_requests_total: Counter<u64>,
    /// Wall-clock time to serve a REST request.
    rest_request_duration: Histogram<f64>,
}

static INSTRUMENTS: LazyLock<Instruments> = LazyLock::new(|| {
    let meter = global::meter("home-gateway");
    Instruments {
        events_total: meter
            .u64_counter("home_gateway_bus_events_total")
            .with_description("Bus events processed by the event dispatcher")
            .build(),
        triggers_total: meter
            .u64_counter("home_gateway_triggers_total")
            .with_description("Trigger evaluations by name and outcome")
            .build(),
        workflows_total: meter
            .u64_counter("home_gateway_workflows_total")
            .with_description("Workflow executions by outcome")
            .build(),
        workflow_duration: meter
            .f64_histogram("home_gateway_workflow_duration_seconds")
            .with_description("Workflow execution duration in seconds")
            .build(),
        steps_total: meter
            .u64_counter("home_gateway_workflow_steps_total")
            .with_description("Workflow step executions by kind and outcome")
            .build(),
        step_duration: meter
            .f64_histogram("home_gateway_workflow_step_duration_seconds")
            .with_description("Workflow step execution duration in seconds")
            .build(),
        device_battery_voltage: meter
            .f64_gauge("home_gateway_device_battery_voltage")
            .with_description("Latest reported device battery voltage")
            .build(),
        device_battery_percent: meter
            .f64_gauge("home_gateway_device_battery_percent")
            .with_description("Latest reported device battery percentage")
            .build(),
        eink_wake_drift: meter
            .f64_histogram("home_gateway_eink_wake_drift_seconds")
            .with_description("Seconds between an eink display's intended wake and its actual poll")
            .build(),
        adhoc_tasks_total: meter
            .u64_counter("home_gateway_adhoc_tasks_total")
            .with_description("Ad-hoc task outcomes by task name and outcome")
            .build(),
        integration_polls_total: meter
            .u64_counter("home_gateway_integration_polls_total")
            .with_description("Integration poll outcomes by integration name and outcome")
            .build(),
        integration_poll_duration: meter
            .f64_histogram("home_gateway_integration_poll_duration_seconds")
            .with_description("Integration poll duration in seconds")
            .build(),
        mqtt_ingest_total: meter
            .u64_counter("home_gateway_mqtt_ingest_total")
            .with_description("MQTT messages ingested, labelled by classified topic kind")
            .build(),
        actor_restarts_total: meter
            .u64_counter("home_gateway_actor_restarts_total")
            .with_description("Supervised actor restarts, labelled by actor and reason")
            .build(),
        device_handler_errors_total: meter
            .u64_counter("home_gateway_device_handler_errors_total")
            .with_description("Device handler message failures, labelled by handler")
            .build(),
        device_last_seen_age: meter
            .f64_gauge("home_gateway_device_last_seen_age_seconds")
            .with_description("Seconds since a watchdog device last reported")
            .build(),
        device_stale: meter
            .u64_gauge("home_gateway_device_stale")
            .with_description("Whether a watchdog device is past its timeout")
            .build(),
        reconciler_retries_total: meter
            .u64_counter("home_gateway_reconciler_retries_total")
            .with_description("Reconciler command re-drives by device kind")
            .build(),
        reconciler_give_ups_total: meter
            .u64_counter("home_gateway_reconciler_give_ups_total")
            .with_description("Reconciler commands abandoned after max_attempts")
            .build(),
        lua_duration: meter
            .f64_histogram("home_gateway_lua_duration_milliseconds")
            .with_description("Lua execution duration in milliseconds by source and outcome")
            .with_boundaries(vec![
                0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0,
            ])
            .build(),
        lua_memory: meter
            .u64_histogram("home_gateway_lua_memory_bytes")
            .with_description(
                "Lua state memory in use when a script finishes, by source and outcome",
            )
            .with_boundaries(vec![
                65_536.0,
                262_144.0,
                1_048_576.0,
                4_194_304.0,
                16_777_216.0,
                33_554_432.0,
                67_108_864.0,
            ])
            .build(),
        lua_vm_setup_duration: meter
            .f64_histogram("home_gateway_lua_vm_setup_duration_milliseconds")
            .with_description("Lua vm creation and api install duration in milliseconds by kind")
            .with_boundaries(vec![
                0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0,
            ])
            .build(),
        rpc_errors_total: meter
            .u64_counter("home_gateway_rpc_errors_total")
            .with_description("Actor rpc failures by target actor and kind")
            .build(),
        db_queries_total: meter
            .u64_counter("home_gateway_db_queries_total")
            .with_description("Repository queries by query name and outcome")
            .build(),
        db_query_duration: meter
            .f64_histogram("home_gateway_db_query_duration_milliseconds")
            .with_description("Repository query duration in milliseconds by query name and outcome")
            .with_boundaries(vec![
                0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0,
            ])
            .build(),
        graphql_operations_total: meter
            .u64_counter("home_gateway_graphql_operations_total")
            .with_description("GraphQL operations by operation name and outcome")
            .build(),
        graphql_operation_duration: meter
            .f64_histogram("home_gateway_graphql_operation_duration_milliseconds")
            .with_description(
                "GraphQL operation duration in milliseconds by operation name and outcome",
            )
            .with_boundaries(vec![
                1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0,
            ])
            .build(),
        rest_requests_total: meter
            .u64_counter("home_gateway_rest_requests_total")
            .with_description("REST requests by method, matched route and status code")
            .build(),
        rest_request_duration: meter
            .f64_histogram("home_gateway_rest_request_duration_milliseconds")
            .with_description(
                "REST request duration in milliseconds by method, matched route and status code",
            )
            .with_boundaries(vec![
                1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0,
            ])
            .build(),
    }
});

pub fn record_rpc_error(actor: &'static str, kind: &'static str) {
    INSTRUMENTS.rpc_errors_total.add(
        1,
        &[KeyValue::new("actor", actor), KeyValue::new("kind", kind)],
    );
}

pub fn record_db_query(query: &'static str, outcome: &'static str, elapsed: Duration) {
    let labels = [
        KeyValue::new("query", query),
        KeyValue::new("outcome", outcome),
    ];

    INSTRUMENTS.db_queries_total.add(1, &labels);
    INSTRUMENTS
        .db_query_duration
        .record(elapsed.as_secs_f64() * 1000.0, &labels);
}

pub fn record_graphql_operation(operation: String, outcome: &'static str, elapsed: Duration) {
    let labels = [
        KeyValue::new("operation", operation),
        KeyValue::new("outcome", outcome),
    ];

    INSTRUMENTS.graphql_operations_total.add(1, &labels);
    INSTRUMENTS
        .graphql_operation_duration
        .record(elapsed.as_secs_f64() * 1000.0, &labels);
}

pub fn record_rest_request(method: String, route: String, status: u16, elapsed: Duration) {
    let labels = [
        KeyValue::new("method", method),
        KeyValue::new("route", route),
        KeyValue::new("status", status.to_string()),
    ];

    INSTRUMENTS.rest_requests_total.add(1, &labels);
    INSTRUMENTS
        .rest_request_duration
        .record(elapsed.as_secs_f64() * 1000.0, &labels);
}

pub fn record_actor_restart(actor: &str, reason: &'static str) {
    INSTRUMENTS.actor_restarts_total.add(
        1,
        &[
            KeyValue::new("actor", actor.to_owned()),
            KeyValue::new("reason", reason),
        ],
    );
}

pub fn record_device_handler_error(handler: &'static str) {
    INSTRUMENTS
        .device_handler_errors_total
        .add(1, &[KeyValue::new("handler", handler)]);
}

pub fn record_integration_poll(name: &'static str, outcome: &'static str, elapsed: Duration) {
    let labels = [
        KeyValue::new("integration", name),
        KeyValue::new("outcome", outcome),
    ];
    INSTRUMENTS.integration_polls_total.add(1, &labels);
    INSTRUMENTS
        .integration_poll_duration
        .record(elapsed.as_secs_f64(), &labels);
}

pub fn record_lua(source: String, outcome: &'static str, elapsed: Duration, memory_bytes: usize) {
    let labels = [
        KeyValue::new("source", source),
        KeyValue::new("outcome", outcome),
    ];

    INSTRUMENTS
        .lua_duration
        .record(elapsed.as_secs_f64() * 1000.0, &labels);
    INSTRUMENTS.lua_memory.record(memory_bytes as u64, &labels);
}

pub fn record_lua_vm_setup(kind: &str, elapsed: Duration) {
    INSTRUMENTS.lua_vm_setup_duration.record(
        elapsed.as_secs_f64() * 1000.0,
        &[KeyValue::new("kind", kind.to_owned())],
    );
}

pub fn record_mqtt_ingest(topic_kind: &'static str) {
    INSTRUMENTS
        .mqtt_ingest_total
        .add(1, &[KeyValue::new("topic_kind", topic_kind)]);
}

pub fn record_adhoc_task(name: &'static str, outcome: &'static str) {
    INSTRUMENTS.adhoc_tasks_total.add(
        1,
        &[
            KeyValue::new("task", name),
            KeyValue::new("outcome", outcome),
        ],
    );
}

pub fn record_eink_wake_drift(device_id: String, drift_secs: f64) {
    INSTRUMENTS
        .eink_wake_drift
        .record(drift_secs, &[KeyValue::new("device_id", device_id)]);
}

pub fn record_device_battery_percent(device_id: String, kind: String, percent: f64) {
    INSTRUMENTS.device_battery_percent.record(
        percent,
        &[
            KeyValue::new("device_id", device_id),
            KeyValue::new("kind", kind),
        ],
    );
}

/// Record the latest battery voltage reported by a poll-transport device.
pub fn record_device_battery_voltage(device_id: String, kind: String, voltage: f64) {
    INSTRUMENTS.device_battery_voltage.record(
        voltage,
        &[
            KeyValue::new("device_id", device_id),
            KeyValue::new("kind", kind),
        ],
    );
}

/// Record how long a watchdog device has been silent, and whether that is stale.
pub fn record_device_liveness(device_key: &str, age_seconds: f64, stale: bool) {
    let labels = [KeyValue::new("device_id", device_key.to_owned())];

    INSTRUMENTS
        .device_last_seen_age
        .record(age_seconds, &labels);
    INSTRUMENTS.device_stale.record(u64::from(stale), &labels);
}

/// The reconciler re-drove an unconfirmed command.
pub fn record_reconciler_retry(kind: &str) {
    INSTRUMENTS
        .reconciler_retries_total
        .add(1, &[KeyValue::new("kind", kind.to_owned())]);
}

/// The reconciler abandoned a command after exhausting its attempts.
pub fn record_reconciler_give_up(kind: &str, device_id: &str) {
    INSTRUMENTS.reconciler_give_ups_total.add(
        1,
        &[
            KeyValue::new("kind", kind.to_owned()),
            KeyValue::new("device_id", device_id.to_owned()),
        ],
    );
}

/// A bus event was received by the dispatcher.
pub fn record_event(event_kind: &'static str) {
    INSTRUMENTS
        .events_total
        .add(1, &[KeyValue::new("event_kind", event_kind)]);
}

/// Outcome of evaluating a trigger against an event.
pub enum TriggerOutcome {
    Fired,
    WhenNotMet,
    CooldownSkipped,
    WhenError,
}

impl TriggerOutcome {
    fn as_str(&self) -> &'static str {
        match self {
            TriggerOutcome::Fired => "fired",
            TriggerOutcome::WhenNotMet => "when_not_met",
            TriggerOutcome::CooldownSkipped => "cooldown_skipped",
            TriggerOutcome::WhenError => "when_error",
        }
    }
}

pub fn record_trigger(name: &str, outcome: TriggerOutcome) {
    INSTRUMENTS.triggers_total.add(
        1,
        &[
            KeyValue::new("trigger", name.to_owned()),
            KeyValue::new("outcome", outcome.as_str()),
        ],
    );
}

/// A workflow finished executing.
pub fn record_workflow(outcome: &'static str, elapsed: Duration) {
    INSTRUMENTS
        .workflows_total
        .add(1, &[KeyValue::new("outcome", outcome)]);
    INSTRUMENTS
        .workflow_duration
        .record(elapsed.as_secs_f64(), &[KeyValue::new("outcome", outcome)]);
}

/// A single workflow step finished executing.
pub fn record_step(kind: &'static str, success: bool, elapsed: Duration) {
    let outcome = if success { "success" } else { "error" };
    let labels = [
        KeyValue::new("step", kind),
        KeyValue::new("outcome", outcome),
    ];
    INSTRUMENTS.steps_total.add(1, &labels);
    INSTRUMENTS
        .step_duration
        .record(elapsed.as_secs_f64(), &labels);
}
