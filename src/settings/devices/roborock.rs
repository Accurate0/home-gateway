use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawRobotVacuumBlock {
    pub name: String,
    pub start_service: String,
    pub stop_service: String,
    pub dock_service: String,
}

#[derive(Debug, Clone)]
pub struct RoborockSettings {
    pub name: String,
    pub control_entity: String,
    pub start_service: String,
    pub stop_service: String,
    pub dock_service: String,
}

impl RawRobotVacuumBlock {
    pub fn resolve(self, address: &str) -> RoborockSettings {
        RoborockSettings {
            name: self.name,
            control_entity: address.to_owned(),
            start_service: self.start_service,
            stop_service: self.stop_service,
            dock_service: self.dock_service,
        }
    }
}
