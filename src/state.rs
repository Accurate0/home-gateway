use sqlx::{Pool, Postgres};

use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::AuthManager;
use crate::device_registry::DeviceRegistry;
use crate::eink::EinkDisplayManager;
use crate::event_bus::EventBus;
use crate::graphql::FinalSchema;
use crate::integrations::feature_flag::FeatureFlagClient;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::jellyfin::Jellyfin;
use crate::integrations::mqtt::MqttClient;
use crate::integrations::s3::S3;
use crate::integrations::transperth::Transperth;
use crate::integrations::willyweather::WillyWeather;
use crate::settings::SettingsContainer;

#[derive(Clone)]
pub struct SharedActorState {
    pub db: Pool<Postgres>,
    pub mqtt: MqttClient,
    pub settings: SettingsContainer,
    pub devices: DeviceRegistry,
    pub feature_flag_client: FeatureFlagClient,
    pub s3: S3,
    pub event_bus: EventBus,
    pub workflows: WorkflowManager,
    pub home_assistant: Option<HomeAssistant>,
    pub jellyfin: Option<Jellyfin>,
    pub transperth: Option<Transperth>,
    pub eink: EinkDisplayManager,
}

#[derive(Clone)]
pub struct ApiState {
    pub feature_flag_client: FeatureFlagClient,
    pub schema: FinalSchema,
    pub settings: SettingsContainer,
    pub db: Pool<Postgres>,
    pub s3: S3,
    pub auth: AuthManager,
    pub devices: DeviceRegistry,
    pub eink: EinkDisplayManager,
    pub willyweather: WillyWeather,
}
