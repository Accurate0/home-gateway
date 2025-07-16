use crate::actors::reminder::cronlike_expression::cronlike_expression_from_str;
use crate::{
    actors::reminder::cronlike_expression::CronlikeExpression,
    timedelta_format::time_delta_from_str,
};
use chrono::{DateTime, FixedOffset, TimeDelta};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::fmt::Display;
use std::{collections::HashMap, path::PathBuf};

pub type IEEEAddress = String;

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
pub struct SynergySettings {
    pub bucket_name: String,
    pub bucket_endpoint: String,
    pub bucket_access_key_id: String,
    pub bucket_access_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database_url: String,
    pub selfbot_api_base: String,
    pub mqtt_url: String,
    pub unifi_api_key: String,
    pub unifi_site_id: String,
    pub unifi_api_base: String,
    pub reminders: Vec<ReminderSettings>,
    pub doors: HashMap<IEEEAddress, DoorSettings>,
    #[serde(rename = "temperatureSensors")]
    pub temperature_sensors: HashMap<IEEEAddress, TemperatureSensorSettings>,
    pub appliances: HashMap<IEEEAddress, ApplianceSettings>,
    pub maccas: MaccasSettings,
    pub discord_token: String,
    pub synergy: SynergySettings,
    pub s3_webhook_secret: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let file_path = PathBuf::from("./config.yaml");
        let file = File::from(file_path).required(false);

        let s = Config::builder()
            .add_source(file)
            .add_source(Environment::default().separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
