use crate::{
    actors::integrations::unifi::{
        UnifiConnectedClientHandler, UnifiMessage, types::UnifiWebhookEvent,
    },
    actors::system::rpc,
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

    if let Err(e) = rpc::cast(
        UnifiConnectedClientHandler::NAME,
        UnifiMessage::Webhook(Box::new(unifi_event)),
    ) {
        tracing::error!("error forwarding unifi event: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::NO_CONTENT
}
