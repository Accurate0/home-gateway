use opentelemetry::{KeyValue, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_resource_detectors::K8sResourceDetector;
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator};
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME,
    TELEMETRY_SDK_VERSION,
};
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() -> anyhow::Result<()> {
    let ingest_url = std::env::var("INGEST_URL").expect("must have INGEST_URL");
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_protocol(Protocol::Grpc)
        .with_endpoint(ingest_url)
        .build()?;

    let tags = vec![
        (KeyValue::new(TELEMETRY_SDK_NAME, "external-tracer".to_string())),
        (KeyValue::new(TELEMETRY_SDK_VERSION, env!("CARGO_PKG_VERSION").to_string())),
        (KeyValue::new(TELEMETRY_SDK_LANGUAGE, "rust".to_string())),
        (KeyValue::new(SERVICE_NAME, format!("home-gateway"))),
        (KeyValue::new(
            DEPLOYMENT_ENVIRONMENT_NAME,
            if cfg!(debug_assertions) {
                "development"
            } else {
                "production"
            },
        )),
    ];

    let resource = Resource::builder()
        .with_attributes(tags)
        .with_detector(Box::new(K8sResourceDetector))
        .build();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(otlp_exporter)
        .build();

    let tracer = tracer_provider.tracer("default");
    opentelemetry::global::set_tracer_provider(tracer_provider);
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    tracing_subscriber::registry()
        .with(
            Targets::default()
                .with_target("otel::tracing", Level::TRACE)
                .with_target("sea_orm::database", Level::TRACE)
                .with_target(
                    "opentelemetry_sdk",
                    if cfg!(debug_assertions) {
                        LevelFilter::OFF
                    } else {
                        LevelFilter::ERROR
                    },
                )
                .with_default(Level::INFO),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;

    Ok(())
}
