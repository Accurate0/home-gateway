use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

fn default_trmnl_refresh() -> TimeDelta {
    TimeDelta::hours(3)
}

fn default_trmnl_base_url() -> String {
    "https://trmnl.com".to_owned()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TrmnlSettings {
    #[serde(default = "default_trmnl_refresh", with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh: TimeDelta,
    #[serde(default = "default_trmnl_base_url")]
    pub base_url: String,
}

impl Default for TrmnlSettings {
    fn default() -> Self {
        Self {
            refresh: default_trmnl_refresh(),
            base_url: default_trmnl_base_url(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawTrmnlBlock {
    pub(crate) name: String,
}

#[derive(Debug, Clone)]
pub struct TrmnlDeviceSettings {
    pub id: String,
    pub name: String,
}
