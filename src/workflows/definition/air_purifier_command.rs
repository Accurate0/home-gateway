use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::repo::air_purifier::AirPurifierMode;
use crate::repo::intent::AirPurifierAttributes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum AirPurifierCommand {
    TurnOn,
    TurnOff,
    SetMode { mode: AirPurifierMode },
    SetSpeed { speed: u8 },
    SetDisplay { on: bool },
}

impl AirPurifierCommand {
    pub fn attributes(&self) -> AirPurifierAttributes {
        match *self {
            AirPurifierCommand::TurnOn => AirPurifierAttributes {
                on: Some(true),
                ..AirPurifierAttributes::default()
            },
            AirPurifierCommand::TurnOff => AirPurifierAttributes {
                on: Some(false),
                ..AirPurifierAttributes::default()
            },
            AirPurifierCommand::SetMode { mode } => AirPurifierAttributes {
                on: Some(true),
                mode: Some(mode),
                ..AirPurifierAttributes::default()
            },
            AirPurifierCommand::SetSpeed { speed } => AirPurifierAttributes {
                on: Some(true),
                mode: Some(AirPurifierMode::Manual),
                speed: Some(i32::from(speed)),
                display: None,
            },
            AirPurifierCommand::SetDisplay { on } => AirPurifierAttributes {
                display: Some(on),
                ..AirPurifierAttributes::default()
            },
        }
    }

    pub fn reapply(attributes: &AirPurifierAttributes) -> Vec<AirPurifierCommand> {
        let mut commands = Vec::new();

        if let Some(on) = attributes.display {
            commands.push(AirPurifierCommand::SetDisplay { on });
        }

        let speed = attributes.speed.and_then(|speed| u8::try_from(speed).ok());

        let power = match (attributes.on, speed, attributes.mode) {
            (Some(false), _, _) => Some(AirPurifierCommand::TurnOff),
            (_, Some(speed), _) => Some(AirPurifierCommand::SetSpeed { speed }),
            (_, None, Some(mode)) => Some(AirPurifierCommand::SetMode { mode }),
            (Some(true), None, None) => Some(AirPurifierCommand::TurnOn),
            (None, None, None) => None,
        };

        commands.extend(power);

        commands
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_speed_command_wants_manual_mode() {
        assert_eq!(
            AirPurifierCommand::SetSpeed { speed: 2 }.attributes(),
            AirPurifierAttributes {
                on: Some(true),
                mode: Some(AirPurifierMode::Manual),
                speed: Some(2),
                display: None,
            }
        );
    }

    #[test]
    fn wanted_attributes_become_the_commands_that_restore_them() {
        let wanted = AirPurifierAttributes {
            on: Some(true),
            mode: Some(AirPurifierMode::Sleep),
            speed: None,
            display: Some(false),
        };

        assert_eq!(
            AirPurifierCommand::reapply(&wanted),
            vec![
                AirPurifierCommand::SetDisplay { on: false },
                AirPurifierCommand::SetMode {
                    mode: AirPurifierMode::Sleep
                },
            ]
        );
    }

    #[test]
    fn a_wanted_off_state_only_turns_off() {
        let wanted = AirPurifierAttributes {
            on: Some(false),
            ..AirPurifierAttributes::default()
        };

        assert_eq!(
            AirPurifierCommand::reapply(&wanted),
            vec![AirPurifierCommand::TurnOff]
        );
    }
}
