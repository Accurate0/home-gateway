use sqlx::{Pool, Postgres};

use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::AuthManager;
use crate::device_registry::DeviceRegistry;
use crate::event_bus::EventBus;
use crate::feature_flag::FeatureFlagClient;
use crate::graphql::FinalSchema;
use crate::home_assistant::HomeAssistant;
use crate::mqtt::MqttClient;
use crate::s3::S3;
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
}
