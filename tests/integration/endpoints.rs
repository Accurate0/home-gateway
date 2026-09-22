use axum::body::Body;
use axum::http::{Request, StatusCode};
use home_gateway::actors::workflows::spawn::spawn_workflows;
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::Harness;

async fn start() -> Harness {
    let harness = Harness::start().await;

    spawn_workflows(&harness.root, harness.state.clone())
        .await
        .expect("failed to spawn the workflow actors");

    harness
}

async fn mint_key(harness: &Harness, scopes: &[&str]) -> String {
    let key = format!("test-key-{}", Uuid::new_v4().simple());
    let hashed = home_gateway::auth::hash_key(&key);
    let scopes: Vec<String> = scopes.iter().map(|scope| (*scope).to_owned()).collect();

    sqlx::query(
        "INSERT INTO api_keys (name, key_prefix, key_hash, scopes) VALUES ($1, $2, $3, $4)",
    )
    .bind(format!("endpoint-test-{}", Uuid::new_v4().simple()))
    .bind(&key[..8])
    .bind(&hashed)
    .bind(&scopes)
    .execute(&harness.db)
    .await
    .expect("failed to insert the test api key");

    key
}

struct Reply {
    status: StatusCode,
    content_type: Option<String>,
    source: Option<String>,
    text: String,
}

impl Reply {
    fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.text).expect("expected a json body")
    }
}

async fn call(harness: &Harness, method: &str, uri: &str, key: Option<&str>, body: &str) -> Reply {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");

    if let Some(key) = key {
        request = request.header("X-Api-Key", key);
    }

    let response = harness
        .router()
        .oneshot(request.body(Body::from(body.to_owned())).unwrap())
        .await
        .expect("the router should not fail");

    let status = response.status();
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    };
    let content_type = header("content-type");
    let source = header("x-source");

    let bytes = response.into_body().collect().await.unwrap().to_bytes();

    Reply {
        status,
        content_type,
        source,
        text: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

async fn get(harness: &Harness, uri: &str, key: &str) -> Reply {
    call(harness, "GET", uri, Some(key), "").await
}

#[tokio::test]
#[serial]
async fn a_returned_table_is_served_as_json() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/plain", &key).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.json(),
        serde_json::json!({ "room": "kitchen", "warm": true })
    );
}

#[tokio::test]
#[serial]
async fn an_envelope_sets_its_status_and_headers() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/envelope", &key).await;

    assert_eq!(reply.status, StatusCode::CREATED);
    assert_eq!(reply.source.as_deref(), Some("lua"));
    assert_eq!(reply.json(), serde_json::json!({ "created": true }));
}

#[tokio::test]
#[serial]
async fn a_string_body_is_sent_verbatim_as_text() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/text", &key).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.text, "pong");
    assert_eq!(
        reply.content_type.as_deref(),
        Some("text/plain; charset=utf-8")
    );
}

#[tokio::test]
#[serial]
async fn a_declared_scope_is_required() {
    let harness = start().await;
    let without = mint_key(&harness, &[]).await;

    let denied = get(&harness, "/v1/test/scoped", &without).await;

    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    assert!(denied.text.contains("light:read"));

    let with = mint_key(&harness, &["light:read"]).await;
    let allowed = get(&harness, "/v1/test/scoped", &with).await;

    assert_eq!(allowed.status, StatusCode::OK);
    assert_eq!(allowed.json(), serde_json::json!({ "allowed": true }));
}

#[tokio::test]
#[serial]
async fn an_endpoint_without_scopes_still_needs_a_key() {
    let harness = start().await;

    let reply = call(&harness, "GET", "/v1/test/plain", None, "").await;

    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn declared_query_parameters_are_coerced() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/params?hours=3&room=kitchen", &key).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.json(),
        serde_json::json!({ "hours": 3, "room": "kitchen" })
    );
}

#[tokio::test]
#[serial]
async fn a_missing_required_parameter_is_a_bad_request() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/params", &key).await;

    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text.contains("`hours` is a required query parameter"));
}

#[tokio::test]
#[serial]
async fn an_uncoercible_parameter_is_a_bad_request() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/params?hours=soon", &key).await;

    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text.contains("`hours` must be a int"));
}

#[tokio::test]
#[serial]
async fn a_path_capture_reaches_the_script() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/rooms/study/summary", &key).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json()["room"], serde_json::json!("study"));
}

#[tokio::test]
#[serial]
async fn a_post_body_arrives_decoded_and_raw() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = call(
        &harness,
        "POST",
        "/v1/test/echo",
        Some(&key),
        r#"{"name":"lamp","count":2}"#,
    )
    .await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json()["name"], serde_json::json!("lamp"));
    assert_eq!(
        reply.json()["raw"],
        serde_json::json!(r#"{"name":"lamp","count":2}"#)
    );
}

#[tokio::test]
#[serial]
async fn a_declared_body_rejects_a_missing_field() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = call(&harness, "POST", "/v1/test/echo", Some(&key), "{}").await;

    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text.contains("`name` is required in the body"));
}

#[tokio::test]
#[serial]
async fn a_declared_body_rejects_malformed_json() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = call(&harness, "POST", "/v1/test/echo", Some(&key), "not json").await;

    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text.contains("not valid json"));
}

#[tokio::test]
#[serial]
async fn an_undeclared_body_keeps_its_arrays() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = call(
        &harness,
        "POST",
        "/v1/test/anything",
        Some(&key),
        r#"{"rooms":["kitchen","study"]}"#,
    )
    .await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.json()["got"]["rooms"],
        serde_json::json!(["kitchen", "study"])
    );
}

#[tokio::test]
#[serial]
async fn the_wrong_method_is_not_allowed() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = call(&harness, "POST", "/v1/test/plain", Some(&key), "{}").await;

    assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
#[serial]
async fn a_failing_script_is_an_internal_error() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/boom", &key).await;

    assert_eq!(reply.status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
#[serial]
async fn a_script_that_runs_too_long_times_out() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/slow", &key).await;

    assert_eq!(reply.status, StatusCode::GATEWAY_TIMEOUT);
}

#[tokio::test]
#[serial]
async fn a_key_without_a_namespace_scope_cannot_see_its_functions() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/light", &key).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json(), serde_json::json!({ "visible": false }));

    let scoped = mint_key(&harness, &["light:read"]).await;
    let reply = get(&harness, "/v1/test/light", &scoped).await;

    assert_eq!(reply.json(), serde_json::json!({ "visible": true }));
}

#[tokio::test]
#[serial]
async fn gw_graphql_runs_a_query() {
    let harness = start().await;
    let key = mint_key(&harness, &["workflow:read"]).await;

    let reply = get(&harness, "/v1/test/graphql", &key).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply.json()["count"].as_i64().is_some());
}

#[tokio::test]
#[serial]
async fn gw_graphql_refuses_a_mutation() {
    let harness = start().await;
    let key = mint_key(&harness, &["lua:write"]).await;

    let reply = get(&harness, "/v1/test/mutation", &key).await;

    assert_eq!(reply.status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
#[serial]
async fn weather_reads_the_stored_forecast() {
    let harness = start().await;
    let key = mint_key(&harness, &["weather:read"]).await;

    let empty = get(&harness, "/v1/test/weather", &key).await;

    assert_eq!(empty.status, StatusCode::OK);
    assert_eq!(empty.json(), serde_json::json!({ "stored": false }));

    sqlx::query(
        "INSERT INTO willyweather_forecast_day
         (location, date, precis_code, precis, min_temp, max_temp, uv_max)
         VALUES ($1, $2, 'fine', 'Sunny', 14, 31, 9.5)",
    )
    .bind("perth")
    .bind(
        chrono::Utc::now()
            .with_timezone(&home_gateway::integrations::willyweather::FORECAST_OFFSET)
            .date_naive(),
    )
    .execute(&harness.db)
    .await
    .expect("failed to seed the forecast");

    let stored = get(&harness, "/v1/test/weather", &key).await;

    assert_eq!(stored.status, StatusCode::OK);
    assert_eq!(stored.json()["stored"], serde_json::json!(true));
    assert_eq!(stored.json()["max"], serde_json::json!(31));
}

#[tokio::test]
#[serial]
async fn weather_rejects_an_unknown_location() {
    let harness = start().await;
    let key = mint_key(&harness, &["weather:read"]).await;

    let reply = get(&harness, "/v1/test/weather-elsewhere", &key).await;

    assert_eq!(reply.status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
#[serial]
async fn weather_requires_its_scope() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/weather", &key).await;

    assert_eq!(reply.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
#[serial]
async fn fuel_returns_the_cheapest_sites_first() {
    let harness = start().await;
    let key = mint_key(&harness, &["fuelwatch:read"]).await;

    for (id, name, price) in [
        (1, "dearest", 199.9),
        (2, "cheapest", 186.7),
        (3, "middle", 190.0),
    ] {
        sqlx::query(
            "INSERT INTO fuelwatch_site
             (site_id, site_name, brand, suburb, postcode, address, price, latitude, longitude)
             VALUES ($1, $2, 'BRAND', 'YANGEBUP', 6164, 'a road', $3, -32.1, 115.8)",
        )
        .bind(id)
        .bind(name)
        .bind(price)
        .execute(&harness.db)
        .await
        .expect("failed to seed a fuel site");
    }

    let reply = get(&harness, "/v1/test/fuel", &key).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.json()["names"],
        serde_json::json!(["cheapest", "middle"])
    );
}

#[tokio::test]
#[serial]
async fn fuel_without_a_postcode_uses_the_configured_one() {
    let harness = start().await;
    let key = mint_key(&harness, &["fuelwatch:read"]).await;

    let reply = get(&harness, "/v1/test/fuel-default", &key).await;

    assert_eq!(reply.status, StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn notify_send_carries_its_actions_and_tag() {
    let harness = start().await;
    let key = mint_key(&harness, &["push:write"]).await;

    let reply = call(&harness, "POST", "/v1/test/notify-actions", Some(&key), "").await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json(), serde_json::json!({ "sent": true }));
}

#[tokio::test]
#[serial]
async fn notify_send_supports_acknowledgement() {
    let harness = start().await;
    let key = mint_key(&harness, &["push:write"]).await;

    let reply = call(
        &harness,
        "POST",
        "/v1/test/notify-acknowledge",
        Some(&key),
        "",
    )
    .await;

    assert_eq!(reply.status, StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn notify_send_rejects_an_unpaired_acknowledge_block() {
    let harness = start().await;
    let key = mint_key(&harness, &["push:write"]).await;

    let reply = call(&harness, "POST", "/v1/test/notify-unpaired", Some(&key), "").await;

    assert_eq!(reply.status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
#[serial]
async fn notify_send_rejects_an_action_naming_an_unknown_workflow() {
    let harness = start().await;
    let key = mint_key(&harness, &["push:write"]).await;

    let reply = call(
        &harness,
        "POST",
        "/v1/test/notify-unknown-workflow",
        Some(&key),
        "",
    )
    .await;

    assert_eq!(reply.status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
#[serial]
async fn gw_graphql_refuses_a_subscription() {
    let harness = start().await;
    let key = mint_key(&harness, &[]).await;

    let reply = get(&harness, "/v1/test/subscription", &key).await;

    assert_eq!(reply.status, StatusCode::INTERNAL_SERVER_ERROR);
}
