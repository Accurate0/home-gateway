use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serial_test::serial;
use tower::ServiceExt;

use crate::common::Harness;

const ENVIRONMENT_ADDRESS: &str = "0x0000000000000002";

struct Client {
    router: axum::Router,
}

impl Client {
    fn new(harness: &Harness) -> Self {
        Self {
            router: harness.router(),
        }
    }

    async fn send(&self, request: Request<Body>) -> (StatusCode, serde_json::Value) {
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("the router should not fail");

        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);

        (status, body)
    }

    async fn graphql(&self, api_key: Option<&str>, query: &str) -> (StatusCode, serde_json::Value) {
        let mut request = Request::builder()
            .method("POST")
            .uri("/v1/graphql")
            .header("content-type", "application/json");

        if let Some(api_key) = api_key {
            request = request.header("X-Api-Key", api_key);
        }

        let body = serde_json::json!({ "query": query }).to_string();

        self.send(request.body(Body::from(body)).unwrap()).await
    }
}

/// Mints a key through the same path the admin API uses, so the test exercises
/// real hashed-key lookup rather than the static config key.
async fn mint_key(harness: &Harness, name: &str, scopes: &[&str]) -> String {
    let key = format!("test-key-{}", uuid::Uuid::new_v4().simple());
    let hashed = home_gateway::auth::hash_key(&key);
    let scopes: Vec<String> = scopes.iter().map(|s| (*s).to_owned()).collect();

    sqlx::query(
        "INSERT INTO api_keys (name, key_prefix, key_hash, scopes) VALUES ($1, $2, $3, $4)",
    )
    .bind(name)
    .bind(&key[..8])
    .bind(&hashed)
    .bind(&scopes)
    .execute(&harness.db)
    .await
    .expect("failed to insert the test api key");

    key
}

#[tokio::test]
#[serial]
async fn health_is_unauthenticated() {
    let harness = Harness::start().await;
    let client = Client::new(&harness);

    let (status, _) = client
        .send(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
#[serial]
async fn requests_without_credentials_are_rejected() {
    let harness = Harness::start().await;
    let client = Client::new(&harness);

    let (status, _) = client.graphql(None, "{ __typename }").await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "the guarded routes should reject anonymous requests"
    );
}

#[tokio::test]
#[serial]
async fn a_valid_api_key_is_accepted() {
    let harness = Harness::start().await;
    let client = Client::new(&harness);
    let key = mint_key(&harness, "test-admin", &["*"]).await;

    let (status, body) = client.graphql(Some(&key), "{ __typename }").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["__typename"], "QueryRoot");
}

#[tokio::test]
#[serial]
async fn a_revoked_api_key_is_rejected() {
    let harness = Harness::start().await;
    let client = Client::new(&harness);
    let key = mint_key(&harness, "test-revoked", &["*"]).await;

    sqlx::query("UPDATE api_keys SET revoked_at = now() WHERE name = $1")
        .bind("test-revoked")
        .execute(&harness.db)
        .await
        .unwrap();

    let (status, _) = client.graphql(Some(&key), "{ __typename }").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn temperature_readings_are_queryable_over_graphql() {
    let harness = Harness::start().await;
    let client = Client::new(&harness);
    let key = mint_key(&harness, "test-reader", &["*"]).await;

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

    // `temperature` resolves through LatestTemperatureDataLoader, so this also
    // covers the dataloader wiring in build_schema.
    let (status, body) = client
        .graphql(
            Some(&key),
            r#"{ environment(id: "test-environment") { id name temperature humidity } }"#,
        )
        .await;

    assert_eq!(status, StatusCode::OK, "graphql errors: {body}");

    let environment = &body["data"]["environment"];

    assert_eq!(environment["name"], "Test Room", "got {body}");
    assert_eq!(environment["temperature"], 19.5);
    assert_eq!(environment["humidity"], 55.0);
}

#[tokio::test]
#[serial]
async fn an_insufficient_scope_is_denied() {
    let harness = Harness::start().await;
    let client = Client::new(&harness);
    let key = mint_key(&harness, "test-narrow", &["rest:epd:read"]).await;

    let (status, body) = client
        .graphql(
            Some(&key),
            r#"{ environment(id: "test-environment") { id } }"#,
        )
        .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "scope denials surface as graphql errors, not http failures"
    );
    assert!(
        body["errors"].is_array(),
        "expected a scope guard error, got {body}"
    );
}
