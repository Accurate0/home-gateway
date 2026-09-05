use crate::{
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
    state::AppState,
};
use axum::{Json, extract::State};
use http::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PushTokenPayload {
    pub token: String,
}

pub async fn push_token(
    State(AppState { ref repos, .. }): State<AppState>,
    Auth(auth): Auth,
    Json(payload): Json<PushTokenPayload>,
) -> StatusCode {
    if auth
        .require(&Scope::new(Resource::IngestHome, Action::Write))
        .is_err()
    {
        return StatusCode::FORBIDDEN;
    }

    let result = repos.push().upsert_token(&payload.token).await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(e) => {
            tracing::error!("failed to register push token: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
