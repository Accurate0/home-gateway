use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct GraphqlSettings {
    pub max_depth: usize,
    pub max_complexity: usize,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub query_timeout: TimeDelta,
}

impl GraphqlSettings {
    pub fn query_timeout(&self) -> Duration {
        self.query_timeout.to_std().unwrap_or_default()
    }
}
