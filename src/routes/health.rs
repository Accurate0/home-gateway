use crate::types::ApiState;
use axum::extract::State;
use http::StatusCode;

pub async fn health(State(ApiState { .. }): State<ApiState>) -> StatusCode {
    StatusCode::NO_CONTENT
}
