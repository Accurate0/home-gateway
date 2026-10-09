use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::repo::air_purifier::AirPurifierMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum AirPurifierCommand {
    TurnOn,
    TurnOff,
    SetMode { mode: AirPurifierMode },
    SetSpeed { speed: u8 },
    SetDisplay { on: bool },
}

impl std::fmt::Display for AirPurifierCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AirPurifierCommand::TurnOn => f.write_str("turn_on"),
            AirPurifierCommand::TurnOff => f.write_str("turn_off"),
            AirPurifierCommand::SetMode { mode } => write!(f, "set_mode {mode}"),
            AirPurifierCommand::SetSpeed { speed } => write!(f, "set_speed {speed}"),
            AirPurifierCommand::SetDisplay { on: true } => f.write_str("set_display on"),
            AirPurifierCommand::SetDisplay { on: false } => f.write_str("set_display off"),
        }
    }
}
