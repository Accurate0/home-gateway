use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

use super::BackoffSettings;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MqttSettings {
    pub url: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub keep_alive: TimeDelta,
    pub max_packet_size: usize,
    pub channel_capacity: usize,
    pub reconnect: BackoffSettings,
}

impl MqttSettings {
    pub fn keep_alive(&self) -> Duration {
        self.keep_alive.to_std().unwrap_or_default()
    }
}
