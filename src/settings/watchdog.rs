use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WatchdogSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub timeout: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub check_interval: TimeDelta,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub realert_after: TimeDelta,
}
