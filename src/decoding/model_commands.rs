use serde::Deserialize;

use crate::settings::VacuumCommands;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCommands {
    pub robot_vacuum: Option<VacuumCommands>,
}
