use crate::{
    actors::unifi::{UnifiConnectedClientHandler, UnifiMessage, types::UnifiWebhookEvent},
    types::ApiState,
};
use axum::{Json, extract::State};
use http::{HeaderMap, StatusCode};

pub async fn unifi(
    State(ApiState { ref settings, .. }): State<ApiState>,
    headers: HeaderMap,
    Json(unifi_event): Json<UnifiWebhookEvent>,
) -> StatusCode {
    let unifi_secret_header = headers.get("X-Webhook-Secret");
    match unifi_secret_header {
        Some(secret_value) if *secret_value == settings.unifi_webhook_secret => {
            let Some(actor) =
                ractor::registry::where_is(UnifiConnectedClientHandler::NAME.to_string())
            else {
                tracing::warn!("unifi actor not found");
                return StatusCode::INTERNAL_SERVER_ERROR;
            };

            if let Err(e) = actor.send_message(UnifiMessage::Webhook(Box::new(unifi_event))) {
                tracing::error!("error forwarding unifi event {e}")
            };

            StatusCode::NO_CONTENT
        }
        _ => StatusCode::UNAUTHORIZED,
    }
}
