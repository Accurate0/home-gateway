use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ts011fPlug1 {
    #[serde(rename = "child_lock")]
    pub child_lock: String,
    pub countdown: i64,
    pub current: f64,
    pub device: Device,
    pub energy: f64,
    #[serde(rename = "indicator_mode")]
    pub indicator_mode: String,
    #[serde(rename = "last_seen")]
    pub last_seen: String,
    pub linkquality: i64,
    pub power: i64,
    #[serde(rename = "power_outage_memory")]
    pub power_outage_memory: String,
    pub state: String,
    pub update: Update,
    pub voltage: i64,
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
    pub stack_version: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub zcl_version: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    #[serde(rename = "installed_version")]
    pub installed_version: i64,
    #[serde(rename = "latest_version")]
    pub latest_version: i64,
    pub state: Option<String>,
}
