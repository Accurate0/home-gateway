use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::SdkMeterProvider,
    propagation::TraceContextPropagator,
    trace::{BatchConfigBuilder, BatchSpanProcessor, Tracer},
};
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME,
    TELEMETRY_SDK_VERSION,
};
use prometheus::Registry;
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

fn telemetry_resource() -> Resource {
    let tags = vec![
        KeyValue::new(TELEMETRY_SDK_NAME, "otel-tracing-rs".to_string()),
        KeyValue::new(TELEMETRY_SDK_VERSION, env!("CARGO_PKG_VERSION").to_string()),
        KeyValue::new(TELEMETRY_SDK_LANGUAGE, "rust".to_string()),
        KeyValue::new(SERVICE_NAME, "home-gateway".to_string()),
        KeyValue::new(
            DEPLOYMENT_ENVIRONMENT_NAME,
            if cfg!(debug_assertions) {
                "development"
            } else {
                "production"
            },
        ),
    ];

    Resource::builder_empty().with_attributes(tags).build()
}

pub fn external_tracer() -> Tracer {
    let ingest_url = std::env::var("OTEL_TRACING_URL").unwrap();

    let resource = telemetry_resource();

    let batch_config = BatchConfigBuilder::default()
        .with_max_queue_size(20480)
        .build();

    let span_exporter = opentelemetry_otlp::HttpExporterBuilder::default()
        .with_protocol(Protocol::HttpJson)
        .with_endpoint(ingest_url)
        .with_timeout(Duration::from_secs(3))
        .build_span_exporter()
        .unwrap();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_span_processor(
            BatchSpanProcessor::builder(span_exporter)
                .with_batch_config(batch_config)
                .build(),
        )
        .with_resource(resource)
        .build();

    let tracer = tracer_provider.tracer("home-gateway");
    global::set_tracer_provider(tracer_provider);

    tracer
}

/// Stand up a pull-based Prometheus metrics pipeline so instruments created via
/// `opentelemetry::global::meter(...)` are actually exported. Without this the
/// SDK uses a no-op provider and every recorded metric is silently dropped.
///
/// Returns the registry to be served from the `/metrics` endpoint that
/// Prometheus (via a ServiceMonitor) scrapes.
pub fn init_metrics() -> Registry {
    let registry = Registry::new();

    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
        .unwrap();

    let meter_provider = SdkMeterProvider::builder()
        .with_reader(exporter)
        .with_resource(telemetry_resource())
        .build();

    global::set_meter_provider(meter_provider);

    registry
}

pub fn init() {
    let tracer = external_tracer();

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    tracing_subscriber::registry()
        .with(
            Targets::default()
                .with_target("otel::tracing", Level::TRACE)
                .with_target("sea_orm::database", Level::TRACE)
                .with_default(Level::INFO),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
}
