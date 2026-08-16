use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AdhocSettings {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub recheck_interval: TimeDelta,
}
