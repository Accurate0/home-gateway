use serde::Deserialize;
use serde::Serialize;

pub type BridgeDevices = Vec<Device>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub disabled: bool,
    #[serde(rename = "friendly_name")]
    pub friendly_name: String,
    #[serde(rename = "ieee_address")]
    pub ieee_address: String,
    #[serde(rename = "interview_completed")]
    pub interview_completed: bool,
    #[serde(rename = "interview_state")]
    pub interview_state: String,
    pub interviewing: bool,
    #[serde(rename = "network_address")]
    pub network_address: i64,
    pub supported: bool,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "date_code")]
    pub date_code: Option<String>,
    pub manufacturer: Option<String>,
    #[serde(rename = "model_id")]
    pub model_id: Option<String>,
    #[serde(rename = "power_source")]
    pub power_source: Option<String>,
    #[serde(rename = "software_build_id")]
    pub software_build_id: Option<String>,
}
