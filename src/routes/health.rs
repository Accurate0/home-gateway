use crate::types::ApiState;
use axum::extract::State;
use http::StatusCode;
use sqlx::Connection;

pub async fn health(State(ApiState { ref db, .. }): State<ApiState>) -> StatusCode {
    let resp = db.acquire().await;

    if resp.is_err() {
        return StatusCode::SERVICE_UNAVAILABLE;
    }

    let is_db_ok = resp.unwrap().ping().await.is_ok();

    if is_db_ok {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
