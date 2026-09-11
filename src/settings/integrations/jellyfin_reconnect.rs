use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct JellyfinReconnectSettings {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub min: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub max: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub stable_after: TimeDelta,
}

impl JellyfinReconnectSettings {
    pub fn min(&self) -> Duration {
        self.min.to_std().unwrap_or_default()
    }

    pub fn max(&self) -> Duration {
        self.max.to_std().unwrap_or_default()
    }

    pub fn stable_after(&self) -> Duration {
        self.stable_after.to_std().unwrap_or_default()
    }
}
