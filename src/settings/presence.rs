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
    pub sensor_type: PresenceSensorType,
    /// Set when the sensor is esphome: the binary_sensor object_id treated as
    /// motion.
    pub motion_entity: Option<String>,
}
