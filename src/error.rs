use axum::response::{IntoResponse, Response};
use http::StatusCode;

use crate::integrations::mqtt::MqttError;
use crate::integrations::woolworths::WoolworthsError;

pub enum AppError {
    Error(anyhow::Error),
    #[allow(unused)]
    StatusCode(StatusCode),
}

impl AppError {
    pub fn message(self) -> anyhow::Error {
        match self {
            AppError::Error(e) => e,
            AppError::StatusCode(status) => anyhow::anyhow!("{status}"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Error(e) => {
                tracing::error!("Something went wrong: {e:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
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

#[derive(thiserror::Error, Debug)]
pub enum MainError {
    #[error(transparent)]
    Mqtt(#[from] MqttError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Woolworths(#[from] WoolworthsError),
    #[error(transparent)]
    WillyWeather(#[from] crate::integrations::willyweather::WillyWeatherError),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
