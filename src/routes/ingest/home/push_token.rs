use crate::types::ApiState;
use axum::{Json, extract::State};
use http::{HeaderMap, StatusCode};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PushTokenPayload {
    pub token: String,
}

/// Register (or refresh) an Android device's FCM token so it can receive pushes.
pub async fn push_token(
    State(ApiState {
        ref settings,
        ref db,
        ..
    }): State<ApiState>,
    headers: HeaderMap,
    Json(payload): Json<PushTokenPayload>,
) -> StatusCode {
    let secret_header = headers.get("X-Webhook-Secret");
    let settings = settings.load();
    match secret_header {
        Some(secret_value) if *secret_value == settings.android_app_webhook_secret => {
            let result = sqlx::query(
                "INSERT INTO android_push_tokens (token) VALUES ($1) \
                 ON CONFLICT (token) DO UPDATE SET updated_at = now()",
            )
            .bind(&payload.token)
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
        _ => StatusCode::UNAUTHORIZED,
    }
}
