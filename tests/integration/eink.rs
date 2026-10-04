use axum::body::Body;
use axum::http::{Request, StatusCode};
use home_gateway::actors::eink_display::{EInkDisplayActor, EInkDisplayMessage};
use home_gateway::eink::EinkDisplayManager;
use home_gateway::eink::manager::source::SourceImage;
use home_gateway::eink::panel::PACKED_FRAME_SIZE;
use home_gateway::grpc::proto::wake_response::Refresh;
use home_gateway::grpc::proto::{WakeRequest, WakeResponse};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use prost::Message;
use ractor::{Actor, ActorProcessingErr, ActorRef};
use serial_test::serial;
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

const WAKE_PATH: &str = "/v1/grpc/home_gateway.eink.v1.EinkDisplay/Wake";
const UNKNOWN_SERVICE_PATH: &str = "/v1/grpc/home_gateway.eink.v1.Nope/Wake";
const PANEL_ADDRESS: &str = "0000000000e1";
const GRPC_UNIMPLEMENTED: i32 = 12;
const GRPC_OK: i32 = 0;
const GRPC_NOT_FOUND: i32 = 5;
const GRPC_PERMISSION_DENIED: i32 = 7;
const MESSAGE_FRAME: u8 = 0x00;
const TRAILERS_FRAME: u8 = 0x80;

struct StubDisplayActor;

impl Actor for StubDisplayActor {
    type Msg = EInkDisplayMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let EInkDisplayMessage::PrepareRender { reply, .. } = message {
            let _ = reply.send(());
        }

        Ok(())
    }
}

async fn stub_display_actor() -> ActorRef<EInkDisplayMessage> {
    let (actor, _) = Actor::spawn(
        Some(EInkDisplayActor::NAME.to_owned()),
        StubDisplayActor,
        (),
    )
    .await
    .expect("failed to spawn the stub display actor");

    actor
}

async fn stop(actor: ActorRef<EInkDisplayMessage>) {
    actor
        .stop_and_wait(None, None)
        .await
        .expect("failed to stop the stub display actor");
}

struct WakeReply {
    http: StatusCode,
    grpc_status: Option<i32>,
    message: Option<WakeResponse>,
}

fn grpc_status_in(trailers: &[u8]) -> Option<i32> {
    String::from_utf8_lossy(trailers)
        .lines()
        .find_map(|line| line.strip_prefix("grpc-status:"))
        .and_then(|status| status.trim().parse().ok())
}

async fn wake(harness: &Harness, key: Option<&str>, request: WakeRequest) -> WakeReply {
    let message = request.encode_to_vec();

    let mut body = vec![MESSAGE_FRAME];
    body.extend_from_slice(&(message.len() as u32).to_be_bytes());
    body.extend_from_slice(&message);

    let mut builder = Request::builder()
        .method("POST")
        .uri(WAKE_PATH)
        .header("Content-Type", "application/grpc-web+proto")
        .header("X-Grpc-Web", "1");

    if let Some(key) = key {
        builder = builder.header("X-Api-Key", key);
    }

    let response = harness
        .router()
        .oneshot(builder.body(Body::from(body)).unwrap())
        .await
        .expect("the router should not fail");

    let http = response.status();

    let mut grpc_status = response
        .headers()
        .get("grpc-status")
        .and_then(|status| status.to_str().ok())
        .and_then(|status| status.parse().ok());

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let mut message = None;
    let mut rest = &body[..];

    while rest.len() >= 5 {
        let len = u32::from_be_bytes([rest[1], rest[2], rest[3], rest[4]]) as usize;
        let payload = &rest[5..5 + len];

        match rest[0] {
            MESSAGE_FRAME => message = Some(WakeResponse::decode(payload).unwrap()),
            TRAILERS_FRAME => grpc_status = grpc_status_in(payload),
            other => panic!("unexpected grpc-web frame flag {other:#x}"),
        }

        rest = &rest[5 + len..];
    }

    WakeReply {
        http,
        grpc_status,
        message,
    }
}

fn wake_request(device_id: &str) -> WakeRequest {
    WakeRequest {
        device_id: device_id.to_owned(),
        battery_voltage: Some(4.0),
        is_charging: false,
        battery_chemistry: "lipo".to_owned(),
        battery_kind: "rechargeable".to_owned(),
        firmware_version: "v0.1.0".to_owned(),
        previous_refresh_failed: false,
        rtc_unix_ms: None,
    }
}

fn wake_request_at(device_id: &str, rtc_unix_ms: i64) -> WakeRequest {
    WakeRequest {
        rtc_unix_ms: Some(rtc_unix_ms),
        ..wake_request(device_id)
    }
}

async fn render(harness: &Harness, image_key: &str, fill: u8) -> String {
    let eink = manager(harness);

    eink.store_render(
        PANEL_ADDRESS,
        "Test Panel",
        &SourceImage {
            image_key: image_key.to_owned(),
            content_hash: image_key.to_owned(),
            payload: None,
        },
    )
    .await
    .unwrap_or_else(|e| panic!("failed to store the render: {}", e.message()));

    let resolved = eink
        .resolve(PANEL_ADDRESS)
        .await
        .expect("the fixture panel should resolve");

    let plan = eink
        .plan(&resolved)
        .await
        .expect("a stored render should plan");

    eink.store_packed(&plan.hash, frame(fill)).await;

    plan.hash
}

async fn displayed_hash(harness: &Harness) -> Option<String> {
    manager(harness).displayed_hash(PANEL_ADDRESS).await
}

#[tokio::test]
#[serial]
async fn the_first_wake_carries_the_full_frame_and_the_next_one_nothing() {
    let harness = Harness::start().await;
    let actor = stub_display_actor().await;
    let key = mint_key(&harness, &["epd:read"]).await;

    let hash = render(&harness, "renders/first.png", 0x21).await;

    let first = wake(&harness, Some(&key), wake_request(PANEL_ADDRESS)).await;

    assert_eq!(first.http, StatusCode::OK);
    assert_eq!(first.grpc_status, Some(GRPC_OK));

    let response = first.message.expect("a wake should answer with a message");

    assert!(response.sleep_secs >= 60);
    assert!(response.firmware.is_none());

    let Some(Refresh::Full(full)) = response.refresh else {
        panic!("the first wake should carry a full frame");
    };

    assert_eq!(full.image.len(), PACKED_FRAME_SIZE);
    assert!(full.image.iter().all(|byte| *byte == 0x21));
    assert_eq!(displayed_hash(&harness).await, Some(hash));

    let second = wake(&harness, Some(&key), wake_request(PANEL_ADDRESS)).await;

    assert_eq!(second.grpc_status, Some(GRPC_OK));
    assert!(
        second.message.unwrap().refresh.is_none(),
        "an unchanged frame must not be sent again"
    );

    stop(actor).await;
}

#[tokio::test]
#[serial]
async fn a_new_render_is_sent_on_the_next_wake() {
    let harness = Harness::start().await;
    let actor = stub_display_actor().await;
    let key = mint_key(&harness, &["epd:read"]).await;

    render(&harness, "renders/first.png", 0x21).await;
    wake(&harness, Some(&key), wake_request(PANEL_ADDRESS)).await;

    let hash = render(&harness, "renders/second.png", 0x43).await;
    let reply = wake(&harness, Some(&key), wake_request(PANEL_ADDRESS)).await;

    let Some(Refresh::Full(full)) = reply.message.unwrap().refresh else {
        panic!("a changed render should carry a full frame");
    };

    assert!(full.image.iter().all(|byte| *byte == 0x43));
    assert_eq!(displayed_hash(&harness).await, Some(hash));

    stop(actor).await;
}

#[tokio::test]
#[serial]
async fn a_failed_refresh_is_sent_the_frame_again() {
    let harness = Harness::start().await;
    let actor = stub_display_actor().await;
    let key = mint_key(&harness, &["epd:read"]).await;

    render(&harness, "renders/first.png", 0x21).await;
    wake(&harness, Some(&key), wake_request(PANEL_ADDRESS)).await;

    let reply = wake(
        &harness,
        Some(&key),
        WakeRequest {
            previous_refresh_failed: true,
            ..wake_request(PANEL_ADDRESS)
        },
    )
    .await;

    assert!(
        matches!(reply.message.unwrap().refresh, Some(Refresh::Full(_))),
        "a display that failed to draw must get the frame again"
    );

    stop(actor).await;
}

#[tokio::test]
#[serial]
async fn a_display_clock_is_set_once_and_again_only_when_it_is_lost() {
    let harness = Harness::start().await;
    let actor = stub_display_actor().await;
    let key = mint_key(&harness, &["epd:read"]).await;

    let silent = wake(&harness, Some(&key), wake_request(PANEL_ADDRESS)).await;

    assert!(
        silent.message.unwrap().set_rtc_unix_ms.is_none(),
        "firmware that reports no clock must not be asked to set one"
    );

    let before = chrono::Utc::now().timestamp_millis();
    let first = wake(&harness, Some(&key), wake_request_at(PANEL_ADDRESS, 12_000)).await;
    let server_time = first
        .message
        .unwrap()
        .set_rtc_unix_ms
        .expect("a never synced display should be given the server time");

    assert!(server_time >= before);

    let second = wake(
        &harness,
        Some(&key),
        wake_request_at(PANEL_ADDRESS, chrono::Utc::now().timestamp_millis()),
    )
    .await;

    assert!(
        second.message.unwrap().set_rtc_unix_ms.is_none(),
        "a display synced within the interval keeps its clock"
    );

    let lost = wake(&harness, Some(&key), wake_request_at(PANEL_ADDRESS, 12_000)).await;

    assert!(
        lost.message.unwrap().set_rtc_unix_ms.is_some(),
        "a clock behind the last sync was lost and must be set again"
    );

    stop(actor).await;
}

#[tokio::test]
#[serial]
async fn an_outdated_firmware_is_pointed_at_the_update() {
    let harness = Harness::start().await;
    let actor = stub_display_actor().await;
    let key = mint_key(&harness, &["epd:read"]).await;

    let reply = wake(
        &harness,
        Some(&key),
        WakeRequest {
            firmware_version: "v0.0.1".to_owned(),
            ..wake_request(PANEL_ADDRESS)
        },
    )
    .await;

    let response = reply.message.unwrap();
    let firmware = response.firmware.expect("an update should be offered");

    assert_eq!(firmware.version, "v0.1.0");
    assert!(
        firmware
            .url
            .ends_with(&format!("/v1/epd/firmware?device_id={PANEL_ADDRESS}"))
    );
    assert!(
        response.refresh.is_none(),
        "a display with no render yet has nothing to draw"
    );

    stop(actor).await;
}

#[tokio::test]
#[serial]
async fn an_unregistered_display_is_not_found() {
    let harness = Harness::start().await;
    let actor = stub_display_actor().await;
    let key = mint_key(&harness, &["epd:read"]).await;

    let reply = wake(&harness, Some(&key), wake_request("ffffffffffff")).await;

    assert_eq!(reply.http, StatusCode::OK);
    assert_eq!(reply.grpc_status, Some(GRPC_NOT_FOUND));
    assert!(reply.message.is_none());

    stop(actor).await;
}

#[tokio::test]
async fn wake_requires_the_epd_scope() {
    let harness = Harness::start().await;
    let key = mint_key(&harness, &["light:read"]).await;

    let reply = wake(&harness, Some(&key), wake_request(PANEL_ADDRESS)).await;

    assert_eq!(reply.grpc_status, Some(GRPC_PERMISSION_DENIED));
    assert!(reply.message.is_none());
}

#[tokio::test]
async fn wake_without_credentials_is_unauthorized() {
    let harness = Harness::start().await;

    let reply = wake(&harness, None, wake_request(PANEL_ADDRESS)).await;

    assert_eq!(reply.http, StatusCode::UNAUTHORIZED);
}

async fn post_empty(harness: &Harness, uri: &str, key: Option<&str>) -> axum::response::Response {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/grpc-web+proto");

    if let Some(key) = key {
        builder = builder.header("X-Api-Key", key);
    }

    harness
        .router()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .expect("the router should not fail")
}

#[tokio::test]
async fn the_grpc_fallback_sits_behind_auth() {
    let harness = Harness::start().await;

    let response = post_empty(&harness, UNKNOWN_SERVICE_PATH, None).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(response.headers().get("grpc-status").is_none());
}

#[tokio::test]
async fn an_unknown_service_under_the_grpc_prefix_is_unimplemented() {
    let harness = Harness::start().await;
    let key = mint_key(&harness, &["epd:read"]).await;

    let response = post_empty(&harness, UNKNOWN_SERVICE_PATH, Some(&key)).await;

    let grpc_status = response
        .headers()
        .get("grpc-status")
        .and_then(|status| status.to_str().ok())
        .and_then(|status| status.parse::<i32>().ok());

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(grpc_status, Some(GRPC_UNIMPLEMENTED));
}

#[tokio::test]
async fn the_grpc_fallback_does_not_leak_outside_its_prefix() {
    let harness = Harness::start().await;

    for uri in ["/home_gateway.eink.v1.EinkDisplay/Wake", "/v1/nope"] {
        let response = post_empty(&harness, uri, None).await;

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "`{uri}` is outside /v1/grpc and must stay a plain 404"
        );
        assert!(response.headers().get("grpc-status").is_none());
    }
}
