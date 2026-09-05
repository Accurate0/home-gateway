use axum::extract::FromRef;
use sqlx::{Pool, Postgres};

use crate::device_registry::DeviceRegistry;
use crate::event_bus::EventBus;
use crate::graphql::FinalSchema;
use crate::integrations::feature_flag::FeatureFlagClient;
use crate::repo::RepoRegistry;
use crate::settings::SettingsContainer;

pub mod handles;

pub use handles::{HandleRegistry, HandleRegistryBuilder};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub repos: RepoRegistry,
    pub settings: SettingsContainer,
    pub devices: DeviceRegistry,
    pub event_bus: EventBus,
    pub feature_flag_client: FeatureFlagClient,
    pub handles: HandleRegistry,
}

#[derive(Clone)]
pub struct ApiState {
    pub schema: FinalSchema,
    pub inner: AppState,
}

impl FromRef<ApiState> for AppState {
    fn from_ref(state: &ApiState) -> Self {
        state.inner.clone()
    }
}

impl FromRef<ApiState> for FinalSchema {
    fn from_ref(state: &ApiState) -> Self {
        state.schema.clone()
    }
}
