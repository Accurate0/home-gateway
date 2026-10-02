use schemars::JsonSchema;
use serde::Deserialize;

use super::SamplingSettings;
use super::trace_link_settings::TraceLinkSettings;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TracingSettings {
    pub sampling: SamplingSettings,
    pub trace_link: TraceLinkSettings,
}
