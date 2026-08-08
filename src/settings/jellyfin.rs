use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct JellyfinSettings {
    pub url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub poll_interval: TimeDelta,
}
