use schemars::JsonSchema;
use serde::Deserialize;

use super::SamplingSettings;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TracingSettings {
    pub sampling: SamplingSettings,
}
