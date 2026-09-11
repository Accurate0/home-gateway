use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct RestartSettings {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub backoff_base: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub backoff_max: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub healthy_after: TimeDelta,
}

impl RestartSettings {
    pub fn backoff_base(&self) -> Duration {
        self.backoff_base.to_std().unwrap_or_default()
    }

    pub fn backoff_max(&self) -> Duration {
        self.backoff_max.to_std().unwrap_or_default()
    }

    pub fn healthy_after(&self) -> Duration {
        self.healthy_after.to_std().unwrap_or_default()
    }
}
