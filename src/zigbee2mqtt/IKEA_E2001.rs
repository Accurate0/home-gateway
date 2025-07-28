use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IKEAE2001 {
    #[serde(rename = "last_seen")]
    pub last_seen: String,
    pub linkquality: i64,
    pub update: Update,
    pub battery: Value,
    pub device: Device,
    pub identify: Value,
    pub action: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    #[serde(rename = "installed_version")]
    pub installed_version: i64,
    #[serde(rename = "latest_version")]
    pub latest_version: i64,
    pub state: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub application_version: i64,
    pub date_code: String,
    pub friendly_name: String,
    pub hardware_version: i64,
    pub ieee_addr: String,
    #[serde(rename = "manufacturerID")]
    pub manufacturer_id: i64,
    pub manufacturer_name: String,
    pub model: String,
    pub network_address: i64,
    pub power_source: String,
    #[serde(rename = "softwareBuildID")]
    pub software_build_id: String,
    pub stack_version: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub zcl_version: i64,
}
