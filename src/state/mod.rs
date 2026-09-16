use sqlx::{Pool, Postgres};

use crate::device_registry::DeviceRegistry;
use crate::event_bus::EventBus;
use crate::graphql::FinalSchema;
use crate::integrations::feature_flag::FeatureFlagClient;
use crate::lua::LuaEngine;
use crate::repo::RepoRegistry;
use crate::settings::SettingsContainer;
use crate::tracing_setup::SamplingControl;

pub mod handles;

pub use handles::{HandleRegistry, HandleRegistryBuilder, RequiredHandle};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub repos: RepoRegistry,
    pub settings: SettingsContainer,
    pub devices: DeviceRegistry,
    pub event_bus: EventBus,
    pub feature_flag_client: FeatureFlagClient,
    pub sampling: SamplingControl,
    pub handles: HandleRegistry,
    pub lua: LuaEngine,
    pub schema: FinalSchema,
}
