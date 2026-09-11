use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct DatabaseSettings {
    pub min_connections: u32,
    pub max_connections: u32,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub slow_statement_threshold: TimeDelta,
}

impl DatabaseSettings {
    pub fn slow_statement_threshold(&self) -> Duration {
        self.slow_statement_threshold.to_std().unwrap_or_default()
    }
}
