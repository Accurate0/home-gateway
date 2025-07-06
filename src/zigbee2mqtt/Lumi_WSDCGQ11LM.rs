use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LumiWSDCGQ11LM {
    pub battery: i64,
    pub device: Device,
    pub humidity: f64,
    pub last_seen: String,
    pub linkquality: i64,
    pub power_outage_count: i64,
    pub pressure: f64,
    pub temperature: f64,
    pub voltage: i64,
}

#[derive(Serialize, Deserialize)]
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
    pub device_type: String,
}
