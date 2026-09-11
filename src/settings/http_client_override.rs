use crate::timedelta_format::option_time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
pub struct HttpClientOverride {
    #[serde(default, with = "option_time_delta_from_str")]
    #[schemars(with = "Option<String>")]
    pub timeout: Option<TimeDelta>,
}
