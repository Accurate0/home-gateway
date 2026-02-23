use crate::types::ApiState;
use axum::extract::State;
use http::StatusCode;

pub async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}
