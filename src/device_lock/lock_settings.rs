use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::timedelta_format::time_delta_from_str;

#[derive(Debug, Clone, PartialEq)]
pub struct LockSettings {
    pub physical: TimeDelta,
    pub excluded: Vec<&'static str>,
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawLockSettings<F> {
    #[serde(deserialize_with = "time_delta_from_str::deserialize")]
    #[schemars(with = "String")]
    physical: TimeDelta,
    excluded: Vec<F>,
}

impl<F: Into<&'static str>> RawLockSettings<F> {
    pub fn resolve(self) -> LockSettings {
        LockSettings {
            physical: self.physical,
            excluded: self.excluded.into_iter().map(Into::into).collect(),
        }
    }
}
