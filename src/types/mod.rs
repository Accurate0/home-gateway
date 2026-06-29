use crate::{
    auth::AuthManager,
    device_registry::DeviceRegistry,
    event_bus::EventBus,
    feature_flag::FeatureFlagClient,
    graphql::FinalSchema,
    mqtt::{MqttClient, MqttError},
    s3::S3,
    settings::SettingsContainer,
    woolworths::WoolworthsError,
};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub mod db;

#[derive(Clone)]
pub struct SharedActorState {
    pub db: Pool<Postgres>,
    pub mqtt: MqttClient,
    pub settings: SettingsContainer,
    pub devices: Arc<DeviceRegistry>,
    pub feature_flag_client: FeatureFlagClient,
    pub s3: S3,
    pub event_bus: EventBus,
}

#[derive(Clone)]
pub struct ApiState {
    pub feature_flag_client: FeatureFlagClient,
    pub schema: FinalSchema,
    pub settings: SettingsContainer,
    pub db: Pool<Postgres>,
    pub s3: S3,
    pub auth: AuthManager,
}

pub enum AppError {
    Error(anyhow::Error),
    #[allow(unused)]
    StatusCode(StatusCode),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Error(e) => {
                tracing::error!("Something went wrong: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Something went wrong: {}", e),
                )
                    .into_response()
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
    Unknown(#[from] anyhow::Error),
}
