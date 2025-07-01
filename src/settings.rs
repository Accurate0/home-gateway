use std::{collections::HashMap, path::PathBuf};

use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

pub type IEEEAddress = String;

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
    #[allow(unused)]
    pub name: String,
    #[serde(flatten)]
    pub armed: ArmedDoorStates,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database_url: String,
    pub mqtt_url: String,
    pub unifi_api_key: String,
    pub unifi_site_id: String,
    pub unifi_api_base: String,
    pub doors: HashMap<IEEEAddress, DoorSettings>,
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
