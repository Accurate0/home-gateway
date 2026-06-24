use crate::{
    actors::alarm::{AlarmActor, AlarmMessage, types::AndroidAppAlarmPayload},
    types::ApiState,
};
use axum::{Json, extract::State};
use http::{HeaderMap, StatusCode};

pub async fn alarm(
    State(ApiState { ref settings, .. }): State<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<AndroidAppAlarmPayload>,
) -> StatusCode {
    let secret_header = headers.get("X-Webhook-Secret");
    let settings = settings.load();
    match secret_header {
        Some(secret_value) if *secret_value == settings.android_app_webhook_secret => {
            let Some(actor) = ractor::registry::where_is(AlarmActor::NAME.to_string()) else {
                tracing::warn!("alarm actor not found");
                return StatusCode::INTERNAL_SERVER_ERROR;
            };

            if let Err(e) = actor.send_message(AlarmMessage::NextAlarm(payload)) {
                tracing::error!("error forwarding alarm event {e}")
            };

            StatusCode::NO_CONTENT
        }
        _ => StatusCode::UNAUTHORIZED,
    }
}
