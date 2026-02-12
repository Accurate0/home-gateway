use crate::actors::reminder::cronlike_expression::cronlike_expression_from_str;
use crate::{
    actors::reminder::cronlike_expression::CronlikeExpression,
    timedelta_format::time_delta_from_str,
};
use arc_swap::{ArcSwap, Guard};
use chrono::{DateTime, FixedOffset, TimeDelta};
use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::fmt::Display;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};
use workflow::ActionSettings;

pub mod workflow;

pub type IEEEAddress = String;
pub type SwitchActionId = String;

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum PresenceActionId {
    PresenceDetected,
    NoPresenceDetected,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum NotifySource {
    Discord {
        #[serde(rename = "channelId")]
        channel_id: u64,
        mentions: Vec<u64>,
    },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum ArmedDoorStates {
    Armed {
        #[serde(with = "time_delta_from_str")]
        timeout: TimeDelta,
    },
    Unarmed,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DoorSettings {
    pub name: String,
    pub id: String,
    #[serde(flatten)]
    pub armed: ArmedDoorStates,
    pub notify: Vec<NotifySource>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApplianceSettings {
    pub name: String,
    pub id: String,
    pub current: ApplianceCurrentThreshold,
    pub notify: Vec<NotifySource>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApplianceCurrentThreshold {
    pub threshold: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TemperatureSensorSettings {
    pub id: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReminderState {
    DryRun,
    Active,
}

impl Display for ReminderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReminderState::DryRun => write!(f, "dryrun"),
            ReminderState::Active => write!(f, "active"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReminderSettings {
    #[allow(unused)]
    pub id: String,
    pub name: String,
    pub state: ReminderState,
    #[serde(rename = "startsOn")]
    pub starts_on: DateTime<FixedOffset>,
    #[serde(with = "cronlike_expression_from_str")]
    pub frequency: CronlikeExpression,
    pub notify: Vec<NotifySource>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MaccasOfferSettings {
    pub match_names: Vec<String>,
    pub notify: Vec<NotifySource>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MaccasSettings {
    pub offers: Vec<MaccasOfferSettings>,
    pub webhook_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SwitchSettings {
    #[allow(unused)]
    pub name: String,
    pub actions: HashMap<SwitchActionId, ActionSettings>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PresenceSettings {
    #[allow(unused)]
    pub name: String,
    pub actions: HashMap<PresenceActionId, ActionSettings>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub api_key: String,
    pub database_url: String,
    pub selfbot_api_base: String,
    pub mqtt_url: String,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub reminders: Vec<ReminderSettings>,
    pub doors: HashMap<IEEEAddress, DoorSettings>,
    #[serde(rename = "temperatureSensors")]
    pub temperature_sensors: HashMap<IEEEAddress, TemperatureSensorSettings>,
    pub appliances: HashMap<IEEEAddress, ApplianceSettings>,
    pub maccas: MaccasSettings,
    pub discord_token: String,
    pub unifi_webhook_secret: String,
    pub android_app_webhook_secret: String,
    pub switches: HashMap<IEEEAddress, SwitchSettings>,
    #[serde(rename = "presenceSensors")]
    pub presence_sensors: HashMap<IEEEAddress, PresenceSettings>,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub object_registry_key_id: String,
    pub object_registry_private_key: String,
}

#[derive(Clone)]
pub struct SettingsContainer {
    inner: Arc<ArcSwap<Settings>>,
}

impl SettingsContainer {
    pub fn new() -> Result<Self, ConfigError> {
        // TODO: fetch config from catalog, falling back to file if not found or other error
        let file_path = PathBuf::from("./config.yaml");
        let file = File::from(file_path).required(true);

        let s = Config::builder()
            .add_source(file)
            .add_source(Environment::default().separator("__"))
            .build()?;

        Ok(Self {
            inner: Arc::new(ArcSwap::new(s.try_deserialize()?)),
        })
    }

    pub fn load_full(&self) -> Arc<Settings> {
        self.inner.load_full()
    }

    pub fn load(&self) -> Guard<Arc<Settings>> {
        self.inner.load()
    }

    #[allow(unused)]
    pub fn reload(&self, new_settings: String) -> Result<(), ConfigError> {
        let new_config = Config::builder()
            .add_source(File::from_str(&new_settings, FileFormat::Yaml))
            .add_source(Environment::default().separator("__"))
            .build()?;

        self.inner.store(Arc::new(new_config.try_deserialize()?));

        Ok(())
    }
}
