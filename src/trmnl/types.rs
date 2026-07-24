use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TrmnlDevicesResponse {
    pub data: Vec<TrmnlDevice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrmnlDevice {
    pub id: i64,
    pub name: String,
    pub friendly_id: String,
    pub mac_address: String,
    #[serde(default)]
    pub battery_voltage: Option<f64>,
    #[serde(default)]
    pub percent_charged: Option<f64>,
}
