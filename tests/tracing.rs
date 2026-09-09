use std::collections::HashMap;

use home_gateway::tracing_context::{inject_current, record_error, set_parent};
use home_gateway::tracing_setup::{
    MQTT_INGEST_SPAN, SampleRatios, SamplingControl, ratio_sampler,
};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{
    InMemorySpanExporter, SdkTracerProvider, SimpleSpanProcessor, SpanData,
};
use rstest::rstest;
use tracing_subscriber::layer::SubscriberExt;

struct Harness {
    provider: SdkTracerProvider,
    exporter: InMemorySpanExporter,
}

impl Harness {
    fn new(control: SamplingControl) -> Self {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_span_processor(SimpleSpanProcessor::new(exporter.clone()))
            .with_sampler(ratio_sampler(control))
            .build();

        Self { provider, exporter }
    }

    fn run(&self, f: impl FnOnce()) -> Vec<SpanData> {
        global::set_text_map_propagator(TraceContextPropagator::new());

        let layer = tracing_opentelemetry::layer().with_tracer(self.provider.tracer("test"));
        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, f);

        self.provider.force_flush().ok();

        self.exporter.get_finished_spans().unwrap()
    }
}

fn by_name(spans: &[SpanData]) -> HashMap<String, SpanData> {
    spans
        .iter()
        .map(|s| (s.name.to_string(), s.clone()))
        .collect()
}

fn always_on() -> SamplingControl {
    let control = SamplingControl::default();
    control.replace(SampleRatios {
        default: 1.0,
        by_span: HashMap::new(),
    });

    control
}

/// The whole point of the change: an event that starts at an MQTT packet and
/// ends in a workflow step must be ONE trace, even though every hop is a
/// separate ractor actor that only receives a `traceparent` string.
#[test]
fn the_ingest_to_workflow_chain_is_a_single_connected_trace() {
    let harness = Harness::new(always_on());

    let spans = harness.run(|| {
        // mqtt_ingest: root of the whole thing
        let ingest = tracing::info_span!(parent: None, "mqtt.ingest", topic = "zigbee2mqtt/door");
        let from_ingest = ingest.in_scope(inject_current);

        // device actor: a different actor, reached by a factory cast
        let device = tracing::info_span!(parent: None, "device.handle", device = "0x00124b");
        set_parent(&device, from_ingest.as_deref());
        let from_device = device.in_scope(inject_current);

        // dispatcher: reached over the event bus
        let trigger = tracing::info_span!(parent: None, "trigger.evaluate", trigger = "lamp on");
        set_parent(&trigger, from_device.as_deref());
        let from_trigger = trigger.in_scope(inject_current);

        // workflow factory: another cast
        let workflow = tracing::info_span!(parent: None, "workflow-worker", workflow = "lamp on");
        set_parent(&workflow, from_trigger.as_deref());
        workflow.in_scope(|| {
            let _step = tracing::info_span!("step: light").entered();
        });
    });

    let named = by_name(&spans);
    assert_eq!(spans.len(), 5, "expected every hop to be exported: {named:?}");

    let ingest = &named["mqtt.ingest"];
    let device = &named["device.handle"];
    let trigger = &named["trigger.evaluate"];
    let workflow = &named["workflow-worker"];
    let step = &named["step: light"];

    let trace_id = ingest.span_context.trace_id();
    for (name, span) in [
        ("device.handle", device),
        ("trigger.evaluate", trigger),
        ("workflow-worker", workflow),
        ("step: light", step),
    ] {
        assert_eq!(
            span.span_context.trace_id(),
            trace_id,
            "{name} landed in a different trace, so the chain is broken"
        );
    }

    assert!(
        !ingest.parent_span_id.to_string().chars().any(|c| c != '0'),
        "mqtt.ingest must be the root"
    );
    assert_eq!(device.parent_span_id, ingest.span_context.span_id());
    assert_eq!(trigger.parent_span_id, device.span_context.span_id());
    assert_eq!(workflow.parent_span_id, trigger.span_context.span_id());
    assert_eq!(step.parent_span_id, workflow.span_context.span_id());
}

/// A span with no incoming traceparent must start its own trace rather than
/// silently attaching to whatever the actor runtime happened to have current.
/// This is the bug that merged seven unrelated solar polls into one trace.
#[test]
fn a_detached_actor_span_does_not_adopt_an_ambient_parent() {
    let harness = Harness::new(always_on());

    let spans = harness.run(|| {
        let ambient = tracing::info_span!("long-lived-actor-context");

        ambient.in_scope(|| {
            let _first = tracing::info_span!(parent: None, "solar-actor").entered();
        });

        ambient.in_scope(|| {
            let _second = tracing::info_span!(parent: None, "solar-actor").entered();
        });
    });

    let polls: Vec<_> = spans
        .iter()
        .filter(|s| s.name == "solar-actor")
        .collect();

    assert_eq!(polls.len(), 2);
    assert_ne!(
        polls[0].span_context.trace_id(),
        polls[1].span_context.trace_id(),
        "two independent polls must not share a trace"
    );
}

#[test]
fn mqtt_ingest_is_sampled_out_but_errors_are_always_kept() {
    let control = SamplingControl::default();
    control.replace(SampleRatios {
        default: 1.0,
        by_span: HashMap::from([(MQTT_INGEST_SPAN.to_owned(), 0.0)]),
    });

    let harness = Harness::new(control);

    let spans = harness.run(|| {
        for _ in 0..20 {
            let _dropped = tracing::info_span!(parent: None, "mqtt.ingest", topic = "t").entered();
        }

        let _kept = tracing::error_span!(
            parent: None,
            "mqtt.ingest.error",
            force_sample = "true",
            otel.status_code = "ERROR",
        )
        .entered();
    });

    let names: Vec<_> = spans.iter().map(|s| s.name.to_string()).collect();

    assert!(
        !names.contains(&"mqtt.ingest".to_owned()),
        "ratio 0.0 should have dropped every ingest span, got {names:?}"
    );
    assert!(
        names.contains(&"mqtt.ingest.error".to_owned()),
        "an error span must survive sampling, got {names:?}"
    );
}

#[test]
fn a_child_of_a_sampled_ingest_is_kept() {
    let control = SamplingControl::default();
    control.replace(SampleRatios {
        default: 1.0,
        by_span: HashMap::from([(MQTT_INGEST_SPAN.to_owned(), 1.0)]),
    });

    let harness = Harness::new(control);

    let spans = harness.run(|| {
        let ingest = tracing::info_span!(parent: None, "mqtt.ingest", topic = "t");
        let carried = ingest.in_scope(inject_current);

        let device = tracing::info_span!(parent: None, "device.handle");
        set_parent(&device, carried.as_deref());
        device.in_scope(|| {});
    });

    let names: Vec<_> = spans.iter().map(|s| s.name.to_string()).collect();

    assert!(names.contains(&"mqtt.ingest".to_owned()), "{names:?}");
    assert!(names.contains(&"device.handle".to_owned()), "{names:?}");
}

#[test]
fn a_recorded_error_sets_the_span_status() {
    let harness = Harness::new(always_on());

    let spans = harness.run(|| {
        let span = tracing::info_span!(
            "device.handle",
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        );
        span.in_scope(|| {});
        record_error(&span, "device rejected the command");
    });

    let span = spans
        .iter()
        .find(|s| s.name == "device.handle")
        .expect("span was exported");

    assert!(
        matches!(span.status, opentelemetry::trace::Status::Error { .. }),
        "expected an error status, got {:?}",
        span.status
    );
}

/// Outbound span names and attributes reach Tempo AND, via Tempo's span-metrics
/// generator, become Prometheus series labels. A credential in a URL therefore
/// ends up in long-lived metric storage, so nothing secret may survive here.
const SECRET: &str = "8f2c1d9ab4e7f60351aa27bcd0e4915f";

#[rstest]
#[case::key_in_path(
    "https://api.willyweather.com.au/v2/8f2c1d9ab4e7f60351aa27bcd0e4915f/locations/14576/weather.json?forecasts=weather&days=7",
    "/v2/{redacted}/locations/{id}/weather.json?{redacted}"
)]
#[case::key_in_query(
    "https://au-journeyplanner.silverrail.io/x/Timetable?ApiKey=8f2c1d9ab4e7f60351aa27bcd0e4915f&format=json",
    "/x/Timetable?{redacted}"
)]
#[case::high_cardinality_id(
    "https://www.woolworths.com.au/apis/ui/product/detail/324461",
    "/apis/ui/product/detail/{id}"
)]
#[case::nothing_sensitive(
    "https://uvdata.arpansa.gov.au/xml/uvvalues.xml",
    "/xml/uvvalues.xml"
)]
fn a_credential_in_a_url_never_reaches_a_span(#[case] raw: &str, #[case] expected: &str) {
    let url = reqwest::Url::parse(raw).unwrap();
    let redacted = home_gateway::http::redacted_path(&url);

    assert_eq!(redacted, expected);
    assert!(
        !redacted.contains(SECRET),
        "credential survived redaction of {raw}: {redacted}"
    );
}

/// A malformed or absent traceparent must leave the span as a healthy root
/// rather than panicking or producing an unroutable parent id.
#[test]
fn a_missing_traceparent_leaves_a_clean_root() {
    let harness = Harness::new(always_on());

    let spans = harness.run(|| {
        let span = tracing::info_span!(parent: None, "device.handle");
        set_parent(&span, None);
        span.in_scope(|| {});

        let bad = tracing::info_span!(parent: None, "device.handle.bad");
        set_parent(&bad, Some("not-a-traceparent"));
        bad.in_scope(|| {});
    });

    for span in &spans {
        assert!(
            span.span_context.is_valid(),
            "{} should still have a valid context",
            span.name
        );
    }

    assert_eq!(spans.len(), 2);
}
