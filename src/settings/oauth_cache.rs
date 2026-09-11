use schemars::JsonSchema;
use serde::Deserialize;

use super::CacheSettings;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct OAuthCacheSettings {
    pub keys: CacheSettings,
    pub userinfo: CacheSettings,
}
