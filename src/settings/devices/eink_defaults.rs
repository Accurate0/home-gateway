use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct EinkDefaults {
    pub reddit_limit: u32,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub settle: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub fallback_refresh: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub min_refresh: TimeDelta,
}

impl EinkDefaults {
    pub fn settle(&self) -> Duration {
        self.settle.to_std().unwrap_or_default()
    }

    pub fn fallback_refresh_secs(&self) -> u32 {
        u32::try_from(self.fallback_refresh.num_seconds().max(0)).unwrap_or(u32::MAX)
    }

    pub fn min_refresh_secs(&self) -> u32 {
        u32::try_from(self.min_refresh.num_seconds().max(0)).unwrap_or(u32::MAX)
    }
}
