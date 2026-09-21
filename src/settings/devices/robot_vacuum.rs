use schemars::JsonSchema;
use serde::Deserialize;

use crate::settings::workflow::VacuumCommand;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawRobotVacuumBlock {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VacuumCommands {
    pub start: String,
    pub stop: String,
    pub dock: String,
}

impl VacuumCommands {
    pub fn get(&self, command: VacuumCommand) -> &str {
        match command {
            VacuumCommand::Start => &self.start,
            VacuumCommand::Stop => &self.stop,
            VacuumCommand::Dock => &self.dock,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VacuumTarget {
    HomeAssistant { entity_id: String },
    Mqtt { address: String },
}

#[derive(Debug, Clone)]
pub struct RobotVacuumSettings {
    pub name: String,
    pub commands: VacuumCommands,
    pub target: VacuumTarget,
}
