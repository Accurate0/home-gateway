use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawRoborockBlock {
    pub name: String,
    pub status_entity: String,
    pub battery_entity: String,
    pub room_entity: String,
    pub control_entity: String,
    pub stop_service: String,
    pub dock_service: String,
}

#[derive(Debug, Clone)]
pub struct RoborockSettings {
    pub name: String,
    pub status_entity: String,
    pub battery_entity: String,
    pub room_entity: String,
    pub control_entity: String,
    pub stop_service: String,
    pub dock_service: String,
}

impl RawRoborockBlock {
    pub fn resolve(self) -> RoborockSettings {
        RoborockSettings {
            name: self.name,
            status_entity: self.status_entity,
            battery_entity: self.battery_entity,
            room_entity: self.room_entity,
            control_entity: self.control_entity,
            stop_service: self.stop_service,
            dock_service: self.dock_service,
        }
    }
}
