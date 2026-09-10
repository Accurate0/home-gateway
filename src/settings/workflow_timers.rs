use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct WorkflowTimerSettings {
    #[serde(deserialize_with = "crate::timedelta_format::signed_time_delta_from_str::deserialize")]
    #[schemars(with = "String")]
    pub catch_up_within: TimeDelta,
}
