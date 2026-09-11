use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct BackoffSettings {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub min: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub max: TimeDelta,
}

impl BackoffSettings {
    pub fn min(&self) -> Duration {
        self.min.to_std().unwrap_or_default()
    }

    pub fn max(&self) -> Duration {
        self.max.to_std().unwrap_or_default()
    }
}
