use std::collections::HashMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use opentelemetry::{
    Context, KeyValue, global,
    trace::{Link, SpanKind, TraceId, TracerProvider},
};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::SdkMeterProvider,
    propagation::TraceContextPropagator,
    trace::{
        BatchConfigBuilder, BatchSpanProcessor, Sampler, SamplingDecision, SamplingResult,
        ShouldSample, Tracer,
    },
};
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT_NAME, K8S_NAMESPACE_NAME, K8S_POD_NAME, SERVICE_INSTANCE_ID,
    SERVICE_NAME, SERVICE_VERSION, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME,
    TELEMETRY_SDK_VERSION,
};
use prometheus::Registry;
use std::time::Duration;
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{Layer, filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

pub const VERSION: &str = env!("HOME_GATEWAY_VERSION");

pub const SQLX_QUERY_TARGET: &str = "sqlx::query";

pub const MQTT_INGEST_SPAN: &str = "mqtt.ingest";
pub const FORCE_SAMPLE: &str = "force_sample";

pub const DEFAULT_SAMPLE_RATIO: f64 = 1.0;
pub const DEFAULT_MQTT_SAMPLE_RATIO: f64 = 0.05;

#[derive(Debug, Clone, PartialEq)]
pub struct SampleRatios {
    pub default: f64,
    pub by_span: HashMap<String, f64>,
}

impl SampleRatios {
    pub fn ratio_for(&self, span: &str) -> f64 {
        self.by_span.get(span).copied().unwrap_or(self.default)
    }
}

impl Default for SampleRatios {
    fn default() -> Self {
        Self {
            default: DEFAULT_SAMPLE_RATIO,
            by_span: HashMap::from([(MQTT_INGEST_SPAN.to_owned(), DEFAULT_MQTT_SAMPLE_RATIO)]),
        }
    }
}

#[derive(Clone, Default)]
pub struct SamplingControl {
    ratios: Arc<ArcSwap<SampleRatios>>,
}

impl SamplingControl {
    pub fn ratios(&self) -> Arc<SampleRatios> {
        self.ratios.load_full()
    }

    pub fn ratio_for(&self, span: &str) -> f64 {
        self.ratios.load().ratio_for(span)
    }

    pub fn replace(&self, ratios: SampleRatios) {
        let previous = self.ratios.swap(Arc::new(ratios));
        let current = self.ratios.load();

        if **current != *previous {
            tracing::info!(
                "trace sampling is now default={} overrides={:?}",
                current.default,
                current.by_span
            );
        }
    }
}

pub fn ratio_sampler(control: SamplingControl) -> Sampler {
    Sampler::ParentBased(Box::new(RatioSampler { control }))
}

#[derive(Clone)]
struct RatioSampler {
    control: SamplingControl,
}

impl RatioSampler {
    fn sampled(ratio: f64, trace_id: TraceId) -> bool {
        if ratio >= 1.0 {
            return true;
        }

        if ratio <= 0.0 {
            return false;
        }

        let upper = u64::from_be_bytes(trace_id.to_bytes()[8..16].try_into().unwrap_or_default());

        upper < (ratio * (u64::MAX as f64)) as u64
    }
}

impl std::fmt::Debug for RatioSampler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RatioSampler")
            .field("ratios", &self.control.ratios())
            .finish()
    }
}

impl ShouldSample for RatioSampler {
    fn should_sample(
        &self,
        _parent_context: Option<&Context>,
        trace_id: TraceId,
        name: &str,
        _span_kind: &SpanKind,
        attributes: &[KeyValue],
        _links: &[Link],
    ) -> SamplingResult {
        let forced = attributes
            .iter()
            .any(|kv| kv.key.as_str() == FORCE_SAMPLE && kv.value.as_str() == "true");

        let decision = if forced || Self::sampled(self.control.ratio_for(name), trace_id) {
            SamplingDecision::RecordAndSample
        } else {
            SamplingDecision::Drop
        };

        SamplingResult {
            decision,
            attributes: Vec::new(),
            trace_state: Default::default(),
        }
    }
}

fn telemetry_resource() -> Resource {
    let tags = vec![
        KeyValue::new(TELEMETRY_SDK_NAME, "otel-tracing-rs".to_string()),
        KeyValue::new(TELEMETRY_SDK_VERSION, env!("CARGO_PKG_VERSION").to_string()),
        KeyValue::new(TELEMETRY_SDK_LANGUAGE, "rust".to_string()),
        KeyValue::new(SERVICE_NAME, "home-gateway".to_string()),
        KeyValue::new(SERVICE_VERSION, VERSION.to_string()),
        KeyValue::new(
            DEPLOYMENT_ENVIRONMENT_NAME,
            if cfg!(debug_assertions) {
                "development"
            } else {
                "production"
            },
        ),
    ];

    let mut resource = Resource::builder_empty().with_attributes(tags);

    if let Ok(pod) = std::env::var("K8S_POD_NAME")
        && !pod.is_empty()
    {
        resource = resource.with_attributes([
            KeyValue::new(K8S_POD_NAME, pod.clone()),
            KeyValue::new(SERVICE_INSTANCE_ID, pod),
        ]);
    }

    if let Ok(namespace) = std::env::var("K8S_NAMESPACE_NAME")
        && !namespace.is_empty()
    {
        resource = resource.with_attributes([KeyValue::new(K8S_NAMESPACE_NAME, namespace)]);
    }

    resource.build()
}

pub fn external_tracer(ingest_url: String, control: SamplingControl) -> Tracer {
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
        .with_sampler(ratio_sampler(control))
        .with_resource(resource)
        .build();

    let tracer = tracer_provider.tracer("home-gateway");
    global::set_tracer_provider(tracer_provider);

    tracer
}

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

pub fn telemetry_filter(exporter_level: LevelFilter) -> Targets {
    Targets::default()
        .with_target("otel::tracing", Level::TRACE)
        .with_target("sea_orm::database", Level::TRACE)
        .with_target("opentelemetry_sdk", exporter_level)
        .with_target("ractor", Level::WARN)
        .with_target("async_graphql::dataloader", Level::WARN)
        .with_target(SQLX_QUERY_TARGET, Level::DEBUG)
        .with_default(Level::INFO)
}

pub fn console_filter() -> Targets {
    Targets::default()
        .with_target(SQLX_QUERY_TARGET, LevelFilter::OFF)
        .with_default(Level::INFO)
}

pub fn init() -> SamplingControl {
    let exporter_level = if cfg!(debug_assertions) {
        LevelFilter::OFF
    } else {
        LevelFilter::from_level(Level::INFO)
    };

    let filter = telemetry_filter(exporter_level);

    let control = SamplingControl::default();

    match std::env::var("OTEL_TRACING_URL") {
        Ok(ingest_url) if !ingest_url.is_empty() => {
            let tracer = external_tracer(ingest_url, control.clone());

            opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().with_filter(console_filter()))
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().with_filter(console_filter()))
                .init();
        }
    }

    control
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace_id(upper: u64) -> TraceId {
        let mut bytes = [0u8; 16];
        bytes[8..16].copy_from_slice(&upper.to_be_bytes());

        TraceId::from_bytes(bytes)
    }

    fn control_with(default: f64, overrides: &[(&str, f64)]) -> SamplingControl {
        let control = SamplingControl::default();
        control.replace(SampleRatios {
            default,
            by_span: overrides
                .iter()
                .map(|(name, ratio)| ((*name).to_owned(), *ratio))
                .collect(),
        });

        control
    }

    fn decide(sampler: &RatioSampler, name: &str, attributes: &[KeyValue], id: u64) -> bool {
        let result = sampler.should_sample(
            None,
            trace_id(id),
            name,
            &SpanKind::Internal,
            attributes,
            &[],
        );

        result.decision == SamplingDecision::RecordAndSample
    }

    #[test]
    fn mqtt_is_sampled_out_of_the_box_while_everything_else_is_kept() {
        let sampler = RatioSampler {
            control: SamplingControl::default(),
        };

        assert!(decide(&sampler, "dispatch_event", &[], u64::MAX));
        assert!(!decide(&sampler, MQTT_INGEST_SPAN, &[], u64::MAX));
        assert!(decide(&sampler, MQTT_INGEST_SPAN, &[], 0));
    }

    #[test]
    fn an_override_only_applies_to_its_own_span_name() {
        let sampler = RatioSampler {
            control: control_with(1.0, &[("noisy", 0.0)]),
        };

        assert!(!decide(&sampler, "noisy", &[], 1));
        assert!(decide(&sampler, "quiet", &[], 1));
    }

    #[test]
    fn a_forced_span_is_kept_regardless_of_ratio() {
        let sampler = RatioSampler {
            control: control_with(0.0, &[]),
        };

        assert!(!decide(&sampler, MQTT_INGEST_SPAN, &[], 1));
        assert!(decide(
            &sampler,
            MQTT_INGEST_SPAN,
            &[KeyValue::new(FORCE_SAMPLE, "true")],
            1
        ));
    }

    #[test]
    fn new_ratios_change_later_decisions_without_rebuilding_the_sampler() {
        let control = control_with(0.0, &[]);
        let sampler = RatioSampler {
            control: control.clone(),
        };

        assert!(!decide(&sampler, MQTT_INGEST_SPAN, &[], 1));

        control.replace(SampleRatios {
            default: 1.0,
            by_span: HashMap::new(),
        });

        assert!(decide(&sampler, MQTT_INGEST_SPAN, &[], u64::MAX));
    }

    #[test]
    fn ractor_internal_spans_are_filtered_out() {
        let filter = telemetry_filter(LevelFilter::INFO);

        assert!(
            !filter.would_enable("ractor::actor", &Level::INFO),
            "ractor's long-lived Actor span must not be recorded: it never closes, so it is never \
             exported, and everything that inherits it lands in an unrooted trace"
        );
        assert!(filter.would_enable("ractor::actor", &Level::WARN));
        assert!(filter.would_enable("home_gateway::actors::workflows", &Level::INFO));
    }

    #[test]
    fn the_service_version_carries_the_build_commit() {
        let (crate_version, sha) = VERSION
            .split_once('-')
            .unwrap_or_else(|| panic!("expected `<version>-<sha>`, got `{VERSION}`"));

        assert_eq!(crate_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(sha.len(), 7, "expected a short sha, got `{sha}`");
        assert!(
            sha.chars().all(|c| c.is_ascii_hexdigit()),
            "expected a hex sha, got `{sha}`"
        );
    }

    #[test]
    fn dataloader_internals_are_filtered_out() {
        let filter = telemetry_filter(LevelFilter::INFO);

        assert!(
            !filter.would_enable("async_graphql::dataloader", &Level::INFO),
            "load_one/load_many/do_load/start_fetch are per-field plumbing, not operations"
        );
        assert!(
            filter.would_enable("async_graphql::graphql", &Level::INFO),
            "the named graphql operation span must survive"
        );
    }

    #[test]
    fn an_unlisted_span_falls_back_to_the_default_ratio() {
        let ratios = SampleRatios {
            default: 0.25,
            by_span: HashMap::from([("listed".to_owned(), 0.75)]),
        };

        assert_eq!(ratios.ratio_for("listed"), 0.75);
        assert_eq!(ratios.ratio_for("unlisted"), 0.25);
    }
}
