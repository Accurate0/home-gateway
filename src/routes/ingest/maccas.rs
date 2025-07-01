use crate::types::ApiState;
use axum::extract::State;
use http::StatusCode;

pub async fn maccas(
    State(ApiState {
        #[allow(unused)]
        ref event_handler,
        ..
    }): State<ApiState>,
    body: String,
) -> StatusCode {
    tracing::info!("{body}");
    StatusCode::OK
}
