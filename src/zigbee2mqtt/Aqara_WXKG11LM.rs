use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AqaraWXKG11LM {
    pub battery: i64,
    #[serde(rename = "device_temperature")]
    pub device_temperature: i64,
    #[serde(rename = "last_seen")]
    pub last_seen: String,
    pub linkquality: i64,
    #[serde(rename = "power_outage_count")]
    pub power_outage_count: i64,
    pub voltage: i64,
    pub action: String,
    pub device: Device,
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
