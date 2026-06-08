use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct EsphomeDiscovery {
    // node name, which is also the MQTT topic prefix for all its entities
    pub name: String,
    pub friendly_name: String,
    #[allow(unused)]
    pub mac: Option<String>,
    #[allow(unused)]
    pub ip: Option<String>,
    #[allow(unused)]
    pub version: Option<String>,
}

pub const TEMPERATURE_OBJECT_ID: &str = "dps310_temperature";
pub const PRESSURE_OBJECT_ID: &str = "dps310_pressure";

pub fn motion_state_topic(node: &str, object_id: &str) -> String {
    format!("{node}/binary_sensor/{object_id}/state")
}

pub fn sensor_state_topic(node: &str, object_id: &str) -> String {
    format!("{node}/sensor/{object_id}/state")
}

pub fn parse_binary_state(payload: &[u8]) -> Option<bool> {
    match payload {
        b"ON" => Some(true),
        b"OFF" => Some(false),
        _ => None,
    }
}

pub fn parse_sensor_state(payload: &[u8]) -> Option<f64> {
    std::str::from_utf8(payload).ok()?.trim().parse().ok()
}
