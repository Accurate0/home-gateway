use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetentionParameters {
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub retention: TimeDelta,
}

impl RetentionParameters {
    pub fn retention_secs(&self) -> f64 {
        self.retention.num_seconds() as f64
    }
}
