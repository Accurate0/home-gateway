use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AuthLockoutSettings {
    pub attempts: u32,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub window: TimeDelta,
    pub capacity: u64,
}

impl AuthLockoutSettings {
    pub fn window(&self) -> Duration {
        self.window.to_std().unwrap_or_default()
    }
}
