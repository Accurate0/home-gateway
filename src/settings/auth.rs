use schemars::JsonSchema;
use serde::Deserialize;

use super::{ApiKeySettings, CacheSettings, OAuthSettings};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AuthSettings {
    pub api_key_cache: CacheSettings,
    #[serde(default)]
    pub api_keys: Vec<ApiKeySettings>,
    #[serde(default)]
    pub oauth: Option<OAuthSettings>,
}
