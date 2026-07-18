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

// esphome sensor object_ids the temperature actor knows how to ingest. Different
// Apollo boards expose different sensors: the MTR-1 reports dps310 temperature /
// pressure, while the PLT-1 reports air temperature / humidity plus an ltr390
// light (lux) and UV index. We subscribe to all of them; boards simply never
// publish the topics they lack.
pub const DPS310_TEMPERATURE_OBJECT_ID: &str = "dps310_temperature";
pub const DPS310_PRESSURE_OBJECT_ID: &str = "dps310_pressure";
pub const AIR_TEMPERATURE_OBJECT_ID: &str = "air_temperature";
pub const AIR_HUMIDITY_OBJECT_ID: &str = "air_humidity";
pub const LTR390_LIGHT_OBJECT_ID: &str = "ltr390_light";
pub const LTR390_UV_INDEX_OBJECT_ID: &str = "ltr390_uv_index";
pub const SHTC3_TEMPERATURE_OBJECT_ID: &str = "temperature";
pub const SHTC3_HUMIDITY_OBJECT_ID: &str = "humidity";
pub const BH1750_ILLUMINANCE_OBJECT_ID: &str = "illuminance";

pub const TEMPERATURE_SENSOR_OBJECT_IDS: &[&str] = &[
    DPS310_TEMPERATURE_OBJECT_ID,
    DPS310_PRESSURE_OBJECT_ID,
    AIR_TEMPERATURE_OBJECT_ID,
    AIR_HUMIDITY_OBJECT_ID,
    LTR390_LIGHT_OBJECT_ID,
    LTR390_UV_INDEX_OBJECT_ID,
    SHTC3_TEMPERATURE_OBJECT_ID,
    SHTC3_HUMIDITY_OBJECT_ID,
    BH1750_ILLUMINANCE_OBJECT_ID,
];

/// What a subscribed esphome state topic maps to. Recorded in the subscription
/// registry when we subscribe (driven by discovery + config) so an incoming
/// state message is routed by an exact topic lookup rather than by re-parsing
/// the topic's shape.
#[derive(Debug, Clone, PartialEq)]
pub enum EsphomeTarget {
    /// A `binary_sensor` state topic — motion, routed to the presence actor.
    Motion { node: String, object_id: String },
    /// A `sensor` state topic — a scalar reading, routed to the plant and/or
    /// environment actor depending on which one(s) claim the node.
    Sensor { node: String, object_id: String },
}

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
