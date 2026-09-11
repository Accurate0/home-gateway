use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

use super::jellyfin_reconnect::JellyfinReconnectSettings;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct JellyfinWebsocketSettings {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub keep_alive: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub silence_timeout: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub sessions_interval: TimeDelta,
    pub reconnect: JellyfinReconnectSettings,
}

impl JellyfinWebsocketSettings {
    pub fn keep_alive(&self) -> Duration {
        self.keep_alive.to_std().unwrap_or_default()
    }

    pub fn silence_timeout(&self) -> Duration {
        self.silence_timeout.to_std().unwrap_or_default()
    }
}
