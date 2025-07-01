use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Phillips9290012573A {
    pub brightness: i64,
    pub color: Color,
    #[serde(rename = "color_mode")]
    pub color_mode: String,
    #[serde(rename = "color_temp")]
    pub color_temp: i64,
    pub device: Device,
    pub effect: Value,
    #[serde(rename = "last_seen")]
    pub last_seen: String,
    pub linkquality: i64,
    #[serde(rename = "power_on_behavior")]
    pub power_on_behavior: String,
    pub state: String,
    pub update: Update,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    pub h: i64,
    pub hue: i64,
    pub s: i64,
    pub saturation: i64,
    pub x: f64,
    pub y: f64,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    #[serde(rename = "installed_version")]
    pub installed_version: i64,
    #[serde(rename = "latest_version")]
    pub latest_version: i64,
    pub state: Option<String>,
}
