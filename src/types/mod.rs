use crate::{
    actors::event_handler,
    bucket::S3BucketAccessor,
    delayqueue::DelayQueueError,
    feature_flag::FeatureFlagClient,
    graphql::FinalSchema,
    mqtt::{MqttClient, MqttError},
    settings::{IEEEAddress, SettingsContainer},
    woolworths::WoolworthsError,
};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use ractor::{ActorRef, factory::FactoryMessage};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub mod db;

#[derive(Clone)]
pub struct SharedActorState {
    pub db: Pool<Postgres>,
    pub mqtt: MqttClient,
    pub settings: SettingsContainer,
    pub bucket_accessor: S3BucketAccessor,
    pub feature_flag_client: FeatureFlagClient,
    pub known_devices_map: Arc<RwLock<HashMap<IEEEAddress, String>>>,
}

#[derive(Clone)]
pub struct ApiState {
    #[allow(unused)]
    pub feature_flag_client: FeatureFlagClient,
    pub schema: FinalSchema,
    pub event_handler: ActorRef<FactoryMessage<(), event_handler::Message>>,
    #[allow(unused)]
    pub settings: SettingsContainer,
    #[allow(unused)]
    pub db: Pool<Postgres>,
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

#[derive(thiserror::Error, Debug)]
pub enum MainError {
    #[error(transparent)]
    Mqtt(#[from] MqttError),
    #[error(transparent)]
    DelayQueue(#[from] DelayQueueError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Woolworths(#[from] WoolworthsError),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
