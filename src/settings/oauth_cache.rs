use std::time::Duration;

use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use super::CacheSettings;
use crate::timedelta_format::time_delta_from_str;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct OAuthCacheSettings {
    pub keys: CacheSettings,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub keys_refresh_cooldown: TimeDelta,
    pub userinfo: CacheSettings,
}

impl OAuthCacheSettings {
    pub fn keys_refresh_cooldown(&self) -> Duration {
        self.keys_refresh_cooldown.to_std().unwrap_or_default()
    }
}
