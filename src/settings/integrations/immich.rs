use crate::settings::enabled_state::EnabledState;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ImmichSettings {
    pub state: EnabledState,
    pub url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    pub candidates: u32,
}
