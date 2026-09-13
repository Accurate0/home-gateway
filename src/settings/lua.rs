use std::path::PathBuf;
use std::time::Duration;

use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::timedelta_format::time_delta_from_str;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LuaSettings {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub timeout: TimeDelta,
    pub max_instructions: u32,
    #[serde(default)]
    pub library: Option<PathBuf>,
    #[serde(default)]
    pub scripts: Option<PathBuf>,
}

impl LuaSettings {
    pub fn timeout(&self) -> Duration {
        self.timeout.to_std().unwrap_or_default()
    }
}

impl Default for LuaSettings {
    fn default() -> Self {
        LuaSettings {
            timeout: TimeDelta::seconds(5),
            max_instructions: 5_000_000,
            library: None,
            scripts: None,
        }
    }
}
