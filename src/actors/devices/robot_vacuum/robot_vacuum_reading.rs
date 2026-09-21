use crate::integrations::mqtt::MqttProtocol;

pub struct RobotVacuumReading {
    pub device_id: String,
    pub protocol: Option<MqttProtocol>,
    pub status: Option<String>,
    pub room: Option<String>,
    pub battery: Option<i64>,
    pub fan_speed: Option<String>,
    pub clean_area: Option<f64>,
    pub clean_count: Option<i32>,
    pub attributes: Option<serde_json::Value>,
}
