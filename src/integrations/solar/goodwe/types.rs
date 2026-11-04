use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub account: String,
    pub pwd: String,
    pub agreement: u8,
    pub is_chinese: bool,
    pub is_local: bool,
}

#[derive(Debug, Deserialize)]
pub struct Envelope<T> {
    pub code: Value,
    #[serde(default)]
    pub description: Option<String>,
    pub data: Option<T>,
}

impl<T> Envelope<T> {
    pub fn is_success(&self) -> bool {
        match &self.code {
            Value::String(code) => code == "00000" || code == "0",
            Value::Number(code) => code.as_i64() == Some(0),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginData {
    pub uid: String,
    pub timestamp: Value,
    pub token: String,
    pub client: String,
    pub version: String,
    pub language: String,
    pub api: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    #[serde(default)]
    pub device_detail_list: Vec<DeviceGroup>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceGroup {
    pub device_type: String,
    #[serde(default)]
    pub status_detail_list: Vec<StatusGroup>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusGroup {
    #[serde(default)]
    pub sn_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelecountingGroup {
    #[serde(default)]
    pub factors: Vec<Factor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factor {
    pub code: String,
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationFlow {
    #[serde(default)]
    pub p_ac: Option<Value>,
    #[serde(flatten)]
    pub rest: serde_json::Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricTimeSeries {
    #[serde(default)]
    pub x_axis: Vec<MetricTime>,
    #[serde(default)]
    pub y_axis: Vec<MetricSeries>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricTime {
    pub x_axi: String,
}

#[derive(Debug, Deserialize)]
pub struct MetricSeries {
    pub code: String,
    #[serde(default)]
    pub data: Vec<Option<f64>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    #[serde(default)]
    pub data_list: Vec<StatisticsItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsItem {
    pub item: String,
    #[serde(default)]
    pub statistics_list: Vec<StatisticsValue>,
}

#[derive(Debug, Deserialize)]
pub struct StatisticsValue {
    pub date: String,
    pub val: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct SolarData {
    pub current_w: f64,
    pub today_kwh: f64,
    pub month_kwh: f64,
    pub total_kwh: f64,
    pub serial_number: String,
    pub flow: StationFlow,
    pub telecounting: Vec<TelecountingGroup>,
}

pub fn numeric(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
}
