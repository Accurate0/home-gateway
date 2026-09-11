use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AdhocSettings {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub recheck_interval: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub task_timeout: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub cron_jitter: TimeDelta,
    pub batch_size: i64,
}

impl AdhocSettings {
    pub fn task_timeout(&self) -> Duration {
        self.task_timeout.to_std().unwrap_or_default()
    }

    pub fn cron_jitter(&self) -> Duration {
        self.cron_jitter.to_std().unwrap_or_default()
    }
}
