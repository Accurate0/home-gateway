use axum::body::Body;
use axum::http::{Request, StatusCode};
use home_gateway::eink::EinkDisplayManager;
use home_gateway::eink::panel::PACKED_FRAME_SIZE;
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::Harness;

const A_HASH: &str = "aa00000000000000000000000000000000000000000000000000000000000001";
const ANOTHER_HASH: &str = "bb00000000000000000000000000000000000000000000000000000000000002";

fn frame(fill: u8) -> Vec<u8> {
    vec![fill; PACKED_FRAME_SIZE]
}

fn manager(harness: &Harness) -> &EinkDisplayManager {
    harness.state.handles.expect::<EinkDisplayManager>()
}

async fn mint_key(harness: &Harness, scopes: &[&str]) -> String {
    let key = format!("test-key-{}", Uuid::new_v4().simple());
    let hashed = home_gateway::auth::hash_key(&key);
    let scopes: Vec<String> = scopes.iter().map(|scope| (*scope).to_owned()).collect();

    sqlx::query(
        "INSERT INTO api_keys (name, key_prefix, key_hash, scopes) VALUES ($1, $2, $3, $4)",
    )
    .bind(format!("eink-test-{}", Uuid::new_v4().simple()))
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
    body: Vec<u8>,
}

async fn get_image(harness: &Harness, uri: &str, key: &str) -> Reply {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("X-Api-Key", key)
        .body(Body::empty())
        .unwrap();

    let response = harness
        .router()
        .oneshot(request)
        .await
        .expect("the router should not fail");

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    Reply {
        status,
        body: body.to_vec(),
    }
}

#[tokio::test]
async fn a_stored_frame_is_served_from_memory_without_s3() {
    let harness = Harness::start().await;
    let eink = manager(&harness);

    eink.store_packed(A_HASH, frame(0x12)).await;

    let cached = eink
        .packed_frame(A_HASH)
        .await
        .expect("the stored frame should come back from the cache");

    assert_eq!(cached.len(), PACKED_FRAME_SIZE);
    assert!(cached.iter().all(|byte| *byte == 0x12));
}

#[tokio::test]
async fn storing_a_frame_does_not_wait_for_the_s3_upload() {
    let harness = Harness::start().await;
    let eink = manager(&harness);

    let started = std::time::Instant::now();
    eink.store_packed(A_HASH, frame(0x34)).await;
    let elapsed = started.elapsed();

    assert!(
        elapsed < std::time::Duration::from_secs(1),
        "store_packed blocked for {elapsed:?}, so the upload is still on the request path"
    );
}

#[tokio::test]
async fn an_uncached_frame_is_not_found_rather_than_fetched_forever() {
    let harness = Harness::start().await;

    let missing = manager(&harness).packed_frame(ANOTHER_HASH).await;

    assert!(missing.is_none());
}

#[tokio::test]
async fn a_malformed_hash_is_rejected_before_any_lookup() {
    let harness = Harness::start().await;

    assert!(
        manager(&harness)
            .packed_frame("../etc/passwd")
            .await
            .is_none()
    );
    assert!(manager(&harness).packed_frame("short").await.is_none());
}

#[tokio::test]
async fn the_image_route_serves_a_cached_frame() {
    let harness = Harness::start().await;
    let key = mint_key(&harness, &["epd:read"]).await;

    manager(&harness).store_packed(A_HASH, frame(0x56)).await;

    let reply = get_image(
        &harness,
        &format!("/v1/epd/image/{A_HASH}?device_id=test-display"),
        &key,
    )
    .await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body.len(), PACKED_FRAME_SIZE);
    assert!(reply.body.iter().all(|byte| *byte == 0x56));
}

#[tokio::test]
async fn the_image_route_crops_to_the_requested_window() {
    let harness = Harness::start().await;
    let key = mint_key(&harness, &["epd:read"]).await;

    manager(&harness).store_packed(A_HASH, frame(0x78)).await;

    let reply = get_image(
        &harness,
        &format!("/v1/epd/image/{A_HASH}?device_id=test-display&x=0&y=0&width=64&height=8"),
        &key,
    )
    .await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body.len(), 64 / 2 * 8);
    assert!(reply.body.iter().all(|byte| *byte == 0x78));
}

#[tokio::test]
async fn the_image_route_rejects_a_hash_that_is_not_a_frame_hash() {
    let harness = Harness::start().await;
    let key = mint_key(&harness, &["epd:read"]).await;

    let reply = get_image(&harness, "/v1/epd/image/nope?device_id=test-display", &key).await;

    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_image_route_requires_the_epd_scope() {
    let harness = Harness::start().await;
    let key = mint_key(&harness, &["light:read"]).await;

    manager(&harness).store_packed(A_HASH, frame(0x9a)).await;

    let reply = get_image(
        &harness,
        &format!("/v1/epd/image/{A_HASH}?device_id=test-display"),
        &key,
    )
    .await;

    assert_eq!(reply.status, StatusCode::FORBIDDEN);
}
