use schemars::JsonSchema;
use serde::Deserialize;

use crate::settings::backoff::BackoffSettings;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct ReconnectSettings {
    #[serde(flatten)]
    pub backoff: BackoffSettings,
    pub log_attempts: u32,
}
