use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LastSeenSettings {
    pub capacity: u64,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub write_interval: TimeDelta,
}

impl LastSeenSettings {
    pub fn write_interval(&self) -> Duration {
        self.write_interval.to_std().unwrap_or_default()
    }
}
