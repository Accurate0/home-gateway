use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SolarSettings {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh: TimeDelta,
    #[serde(default)]
    pub goodwe_username: Option<String>,
    #[serde(default)]
    pub goodwe_password: Option<String>,
    #[serde(default)]
    pub goodwe_powerstation_id: Option<String>,
}
