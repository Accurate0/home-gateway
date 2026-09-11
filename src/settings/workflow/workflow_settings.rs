use crate::settings::CacheSettings;
use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

use super::WorkflowTimerSettings;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WorkflowSettings {
    pub workers: usize,
    pub enabled_cache: CacheSettings,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub condition_timeout: TimeDelta,
    pub timers: WorkflowTimerSettings,
}

impl WorkflowSettings {
    pub fn condition_timeout(&self) -> Duration {
        self.condition_timeout.to_std().unwrap_or_default()
    }
}
