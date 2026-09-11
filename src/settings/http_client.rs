use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct HttpClientSettings {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub timeout: TimeDelta,
}

impl HttpClientSettings {
    pub fn timeout(&self) -> Duration {
        self.timeout.to_std().unwrap_or_default()
    }
}
