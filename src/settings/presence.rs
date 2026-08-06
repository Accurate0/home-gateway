use schemars::JsonSchema;
use serde::Deserialize;

use super::de::de_string_or_vec;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum PresenceSensorType {
    #[default]
    Zigbee,
    Esphome,
}

#[derive(Debug, Clone)]
pub struct PresenceSettings {
    #[allow(unused)]
    pub name: String,
    #[allow(unused)]
    pub sensor_type: PresenceSensorType,
    #[allow(unused)]
    pub motion_entities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawPresenceBlock {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default, deserialize_with = "de_string_or_vec")]
    #[schemars(with = "Vec<String>")]
    pub(crate) motion_entity: Vec<String>,
}
