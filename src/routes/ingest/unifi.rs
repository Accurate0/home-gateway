use crate::{
    actors::unifi::{UnifiConnectedClientHandler, UnifiMessage, types::UnifiWebhookEvent},
    auth::{Auth, scope::required},
};
use axum::Json;
use http::StatusCode;

pub async fn unifi(Auth(auth): Auth, Json(unifi_event): Json<UnifiWebhookEvent>) -> StatusCode {
    if auth.require(&required::INGEST_UNIFI_WRITE).is_err() {
        return StatusCode::FORBIDDEN;
    }

    let Some(actor) = ractor::registry::where_is(UnifiConnectedClientHandler::NAME.to_string())
    else {
        tracing::warn!("unifi actor not found");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    if let Err(e) = actor.send_message(UnifiMessage::Webhook(Box::new(unifi_event))) {
        tracing::error!("error forwarding unifi event {e}")
    };

    StatusCode::NO_CONTENT
}
