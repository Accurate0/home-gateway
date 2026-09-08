use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::timedelta_format::time_delta_from_str;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReconcilerSettings {
    pub enabled: bool,
    pub workers: usize,
    #[serde(deserialize_with = "time_delta_from_str::deserialize")]
    #[schemars(with = "String")]
    pub interval: TimeDelta,
    #[serde(deserialize_with = "time_delta_from_str::deserialize")]
    #[schemars(with = "String")]
    pub grace: TimeDelta,
    #[serde(deserialize_with = "time_delta_from_str::deserialize")]
    #[schemars(with = "String")]
    pub backoff: TimeDelta,
    #[serde(deserialize_with = "time_delta_from_str::deserialize")]
    #[schemars(with = "String")]
    pub confirm_timeout: TimeDelta,
    pub max_attempts: i32,
    pub batch_size: i64,
}
