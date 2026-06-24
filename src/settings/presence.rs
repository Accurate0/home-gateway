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
    pub motion_entity: Option<String>,
}
