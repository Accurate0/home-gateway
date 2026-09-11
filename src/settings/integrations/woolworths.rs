use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

fn default_woolworths_refresh() -> TimeDelta {
    TimeDelta::hours(1)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WoolworthsSettings {
    #[serde(default = "default_woolworths_refresh", with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh: TimeDelta,
}

impl Default for WoolworthsSettings {
    fn default() -> Self {
        Self {
            refresh: default_woolworths_refresh(),
        }
    }
}
