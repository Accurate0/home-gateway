use crate::{
    actors::integrations::unifi::{
        UnifiConnectedClientHandler, UnifiMessage, types::UnifiWebhookEvent,
    },
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
};
use axum::Json;
use http::StatusCode;

pub async fn unifi(Auth(auth): Auth, Json(unifi_event): Json<UnifiWebhookEvent>) -> StatusCode {
    if auth
        .require(&Scope::new(Resource::IngestUnifi, Action::Write))
        .is_err()
    {
        return StatusCode::FORBIDDEN;
    }

    let Some(actor) = ractor::registry::where_is(UnifiConnectedClientHandler::NAME) else {
        tracing::warn!("unifi actor not found");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    if let Err(e) = actor.send_message(UnifiMessage::Webhook(Box::new(unifi_event))) {
        tracing::error!("error forwarding unifi event {e}")
    };

    StatusCode::NO_CONTENT
}
