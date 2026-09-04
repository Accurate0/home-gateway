use crate::{
    auth::{
        Auth,
        scope::{Action, Resource, Scope},
    },
    state::ApiState,
};
use axum::{Json, extract::State};
use http::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PushTokenPayload {
    pub token: String,
}

pub async fn push_token(
    State(ApiState { ref db, .. }): State<ApiState>,
    Auth(auth): Auth,
    Json(payload): Json<PushTokenPayload>,
) -> StatusCode {
    if auth
        .require(&Scope::new(Resource::IngestHome, Action::Write))
        .is_err()
    {
        return StatusCode::FORBIDDEN;
    }

    let result = sqlx::query!(
        "INSERT INTO android_push_tokens (token) VALUES ($1) \
         ON CONFLICT (token) DO UPDATE SET updated_at = now()",
        payload.token
    )
    .execute(db)
    .await;

    match result {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(e) => {
            tracing::error!("failed to register push token: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
