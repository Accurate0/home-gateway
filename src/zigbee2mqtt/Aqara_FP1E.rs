use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AqaraFP1E {
    #[serde(rename = "detection_range")]
    pub detection_range: i64,
    pub device: Device,
    #[serde(rename = "device_temperature")]
    pub device_temperature: i64,
    pub identify: Value,
    #[serde(rename = "last_seen")]
    pub last_seen: String,
    pub linkquality: i64,
    #[serde(rename = "motion_sensitivity")]
    pub motion_sensitivity: String,
    pub movement: String,
    #[serde(rename = "power_outage_count")]
    pub power_outage_count: i64,
    pub presence: bool,
    #[serde(rename = "restart_device")]
    pub restart_device: Value,
    #[serde(rename = "spatial_learning")]
    pub spatial_learning: Value,
    #[serde(rename = "target_distance")]
    pub target_distance: f64,
    pub update: Update,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub application_version: i64,
    pub friendly_name: String,
    pub ieee_addr: String,
    #[serde(rename = "manufacturerID")]
    pub manufacturer_id: i64,
    pub manufacturer_name: String,
    pub model: String,
    pub network_address: i64,
    pub power_source: String,
    #[serde(rename = "type")]
    pub type_field: String,
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
