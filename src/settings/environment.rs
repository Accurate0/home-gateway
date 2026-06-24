use crate::esphome::TEMPERATURE_SENSOR_OBJECT_IDS;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum EnvironmentSensorType {
    #[default]
    Zigbee,
    Esphome,
}

/// esphome sensor object_ids to subscribe to when none are configured. Matches
/// the readings the environment actor knows how to ingest; boards simply never
/// publish the topics they lack.
pub(crate) fn default_environment_entities() -> Vec<String> {
    TEMPERATURE_SENSOR_OBJECT_IDS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[derive(Debug, Clone)]
pub struct EnvironmentSensorSettings {
    pub id: String,
    #[allow(unused)]
    pub sensor_type: EnvironmentSensorType,
    /// esphome sensor object_ids to subscribe to on discovery. Unused for zigbee
    /// sources (their readings arrive on the `zigbee2mqtt/+` wildcard).
    #[allow(unused)]
    pub entities: Vec<String>,
}
