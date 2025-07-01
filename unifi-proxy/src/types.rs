use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[derive(Clone)]
pub struct AppState {
    pub unifi_api_base: String,
    pub client: reqwest::Client,
}

pub enum AppError {
    Error(anyhow::Error),
    #[allow(unused)]
    StatusCode(StatusCode),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Error(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", e),
            )
                .into_response(),
            AppError::StatusCode(s) => {
                (s, s.canonical_reason().unwrap_or("").to_owned()).into_response()
            }
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Error(err.into())
    }
}
