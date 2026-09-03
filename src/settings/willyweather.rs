use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WillyWeatherSettings {
    #[serde(default)]
    pub api_key: Option<String>,
    pub default_location: String,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub cache_ttl: TimeDelta,
}
