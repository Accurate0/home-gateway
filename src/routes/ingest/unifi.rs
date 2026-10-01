use crate::{
    actors::integrations::unifi::{
        UnifiConnectedClientHandler, UnifiMessage, types::UnifiWebhookEvent,
    },
    actors::system::rpc,
    auth::{
        AuthContext,
        scope::{Action, Resource},
    },
    error::AppError,
};
use anyhow::Context;
use axum::Json;
use http::StatusCode;

pub async fn unifi(
    auth: AuthContext,
    Json(unifi_event): Json<UnifiWebhookEvent>,
) -> Result<StatusCode, AppError> {
    auth.require(Resource::IngestUnifi, Action::Write)?;

    rpc::cast(
        UnifiConnectedClientHandler::NAME,
        UnifiMessage::Webhook(Box::new(unifi_event)),
    )
    .context("forwarding unifi event")?;

    Ok(StatusCode::NO_CONTENT)
}
