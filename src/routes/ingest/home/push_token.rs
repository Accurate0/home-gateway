use crate::{
    auth::{
        AuthContext,
        scope::{Action, Resource},
    },
    error::AppError,
    state::AppState,
};
use anyhow::Context;
use axum::{Json, extract::State};
use http::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PushTokenPayload {
    pub token: String,
}

pub async fn push_token(
    State(AppState { ref repos, .. }): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<PushTokenPayload>,
) -> Result<StatusCode, AppError> {
    auth.require(Resource::IngestHome, Action::Write)?;

    repos
        .push()
        .upsert_token(&payload.token)
        .await
        .context("registering push token")?;

    Ok(StatusCode::NO_CONTENT)
}
