use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

fn default_postcode() -> i32 {
    6000
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FuelWatchSettings {
    #[serde(default = "default_postcode")]
    pub postcode: i32,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh: TimeDelta,
}
