use std::{collections::HashMap, path::PathBuf};

use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

pub type IEEEAddress = String;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum NotifySource {
    Discord {
        #[serde(rename = "channelId")]
        channel_id: i64,
        mentions: Vec<i64>,
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
pub struct Settings {
    pub database_url: String,
    pub selfbot_api_base: String,
    pub mqtt_url: String,
    pub unifi_api_key: String,
    pub unifi_site_id: String,
    pub unifi_api_base: String,
    pub doors: HashMap<IEEEAddress, DoorSettings>,
    #[serde(rename = "temperatureSensors")]
    pub temperature_sensors: HashMap<IEEEAddress, TemperatureSensorSettings>,
    pub appliances: HashMap<IEEEAddress, ApplianceSettings>,
    pub maccas: MaccasSettings,
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
