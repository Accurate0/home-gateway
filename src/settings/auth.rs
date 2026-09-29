use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

use super::{ApiKeySettings, AuthLockoutSettings, CacheSettings, OAuthSettings};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AuthSettings {
    pub api_key_cache: CacheSettings,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub last_used_interval: TimeDelta,
    pub lockout: AuthLockoutSettings,
    #[serde(default)]
    pub api_keys: Vec<ApiKeySettings>,
    #[serde(default)]
    pub oauth: Option<OAuthSettings>,
}

impl AuthSettings {
    pub fn last_used_interval(&self) -> Duration {
        self.last_used_interval.to_std().unwrap_or_default()
    }
}
