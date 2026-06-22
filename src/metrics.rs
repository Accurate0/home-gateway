//! Domain metrics for trigger dispatch and workflow execution.
//!
//! Mirrors the janitor-bot pattern: a small module of `record_*` helpers that
//! the dispatcher and workflow executor call, keeping instrument names and
//! label conventions in one place. Instruments are created lazily against the
//! global meter set up in [`crate::tracing_setup::init_metrics`], so they're
//! exported via the same Prometheus `/metrics` endpoint.

use std::sync::LazyLock;
use std::time::Duration;

use opentelemetry::metrics::{Counter, Histogram};
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
    }
});

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
