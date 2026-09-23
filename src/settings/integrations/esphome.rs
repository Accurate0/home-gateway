use std::path::PathBuf;
use std::time::Duration;

use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::settings::enabled_state::EnabledState;
use crate::timedelta_format::time_delta_from_str;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct EsphomeSettings {
    pub state: EnabledState,
    pub models: PathBuf,
    pub port: u16,
    #[serde(default)]
    pub encryption_key: Option<String>,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub keep_alive: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub silence_timeout: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub reconnect_delay: TimeDelta,
}

impl EsphomeSettings {
    pub fn keep_alive(&self) -> Duration {
        self.keep_alive.to_std().unwrap_or_default()
    }

    pub fn silence_timeout(&self) -> Duration {
        self.silence_timeout.to_std().unwrap_or_default()
    }

    pub fn reconnect_delay(&self) -> Duration {
        self.reconnect_delay.to_std().unwrap_or_default()
    }
}
