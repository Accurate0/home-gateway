use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CacheSettings {
    pub capacity: u64,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub ttl: TimeDelta,
}

impl CacheSettings {
    pub fn ttl(&self) -> Duration {
        self.ttl.to_std().unwrap_or_default()
    }
}
