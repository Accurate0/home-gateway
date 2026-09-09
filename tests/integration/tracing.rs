use home_gateway::auth::context::AuthContext;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::{
    InMemorySpanExporter, SdkTracerProvider, SimpleSpanProcessor, SpanData,
};
use serial_test::serial;
use tracing::instrument::WithSubscriber;
use tracing_subscriber::layer::SubscriberExt;

use crate::common::Harness;

const ENVIRONMENT_ADDRESS: &str = "0x0000000000000002";

async fn spans_for<F, Fut>(f: F) -> Vec<SpanData>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let exporter = InMemorySpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_span_processor(SimpleSpanProcessor::new(exporter.clone()))
        .build();

    let layer = tracing_opentelemetry::layer().with_tracer(provider.tracer("test"));
    let subscriber = tracing_subscriber::registry()
        .with(home_gateway::tracing_setup::telemetry_filter(
            tracing_subscriber::filter::LevelFilter::INFO,
        ))
        .with(layer);

    f().with_subscriber(subscriber).await;

    provider.force_flush().ok();

    exporter.get_finished_spans().unwrap()
}

fn request(query: &str) -> async_graphql::Request {
    async_graphql::Request::new(query).data(AuthContext::full_access(false))
}

fn names(spans: &[SpanData]) -> Vec<String> {
    spans.iter().map(|s| s.name.to_string()).collect()
}

fn attribute(span: &SpanData, key: &str) -> Option<String> {
    span.attributes
        .iter()
        .find(|kv| kv.key.as_str() == key)
        .map(|kv| kv.value.as_str().to_string())
}

async fn seed(harness: &Harness) {
    sqlx::query(
        "INSERT INTO latest_temperature_sensor (name, entity_id, ieee_addr, temperature, humidity)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind("Test Room")
    .bind("test-room")
    .bind(ENVIRONMENT_ADDRESS)
    .bind(19.5_f64)
    .bind(55.0_f64)
    .execute(&harness.db)
    .await
    .expect("failed to seed a temperature reading");
}

#[tokio::test]
#[serial]
async fn a_graphql_query_produces_one_span_per_top_level_field() {
    let harness = Harness::start().await;
    seed(&harness).await;

    let schema = harness.api_state().schema;

    let spans = spans_for(|| async move {
        let response = schema
            .execute(request(
                r#"query DashboardEntities {
                    environment(id: "test-environment") { id name temperature humidity }
                }"#,
            ))
            .await;

        assert!(response.errors.is_empty(), "{:?}", response.errors);
    })
    .await;

    let names = names(&spans);

    assert!(
        names.contains(&"graphql DashboardEntities".to_owned()),
        "the operation span must carry the operation name: {names:?}"
    );
    assert!(names.contains(&"parse_query".to_owned()), "{names:?}");
    assert!(names.contains(&"validation".to_owned()), "{names:?}");

    let field_spans: Vec<_> = names
        .iter()
        .filter(|n| n.starts_with("QueryRoot."))
        .collect();
    assert_eq!(
        field_spans,
        vec!["QueryRoot.environment"],
        "exactly one span for the one top-level field: {names:?}"
    );

    let nested: Vec<_> = names
        .iter()
        .filter(|n| {
            n.starts_with("EnvironmentEntity.") || n.starts_with('[') || n.contains("].")
        })
        .collect();
    assert!(
        nested.is_empty(),
        "leaf and list-item resolvers must not each get a span: {nested:?}"
    );
}

#[tokio::test]
#[serial]
async fn the_parse_span_records_the_operation_and_its_top_level_fields() {
    let harness = Harness::start().await;
    seed(&harness).await;

    let schema = harness.api_state().schema;

    let spans = spans_for(|| async move {
        schema
            .execute(request(
                r#"query DashboardEntities {
                    environment(id: "test-environment") { id temperature }
                }"#,
            ))
            .await;
    })
    .await;

    let parse = spans
        .iter()
        .find(|s| s.name == "parse_query")
        .expect("parse_query span");

    assert_eq!(
        attribute(parse, "operation").as_deref(),
        Some("DashboardEntities")
    );
    assert_eq!(attribute(parse, "fields").as_deref(), Some("environment"));
}

#[tokio::test]
#[serial]
async fn a_bulk_query_span_reports_its_batch_size() {
    let harness = Harness::start().await;
    seed(&harness).await;

    let repos = harness.state.repos.clone();

    let spans = spans_for(|| async move {
        repos
            .environment()
            .latest_many(&[
                "test-room".to_owned(),
                "other-room".to_owned(),
                "third-room".to_owned(),
            ])
            .await
            .expect("the bulk read should succeed");
    })
    .await;

    let loader = spans
        .iter()
        .find(|s| s.name == "bulk-get-temperature")
        .unwrap_or_else(|| panic!("no dataloader span; got {:?}", names(&spans)));

    assert_eq!(
        attribute(loader, "keys").as_deref(),
        Some("3"),
        "the batch size is what tells you whether the dataloader batched"
    );
}

#[tokio::test]
#[serial]
async fn dataloader_and_actor_internals_stay_out_of_the_trace() {
    let harness = Harness::start().await;
    seed(&harness).await;

    let schema = harness.api_state().schema;

    let spans = spans_for(|| async move {
        schema
            .execute(request(
                r#"{ environment(id: "test-environment") { id temperature } }"#,
            ))
            .await;
    })
    .await;

    let names = names(&spans);

    for noise in ["load_one", "load_many", "start_fetch", "do_load", "Actor"] {
        assert!(
            !names.contains(&noise.to_owned()),
            "{noise} is library plumbing and must be filtered out: {names:?}"
        );
    }
}
