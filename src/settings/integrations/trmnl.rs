use crate::settings::enabled_state::EnabledState;
use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TrmnlSettings {
    pub state: EnabledState,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh: TimeDelta,
    pub base_url: String,
}
