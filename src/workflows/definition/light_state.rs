use schemars::JsonSchema;
use serde::Deserialize;

use crate::device_registry::Capability;

/// Brightness / colour-temperature mutations applied to a light. Kept in
/// `SCREAMING_SNAKE_CASE` to match the long-standing on-disk config.
#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(tag = "state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LightState {
    On,
    Off,
    Toggle,
    SetBrightness {
        value: u64,
    },
    IncreaseBrightness {
        value: u64,
        #[serde(default)]
        on_off: bool,
    },
    DecreaseBrightness {
        value: u64,
        #[serde(default)]
        on_off: bool,
    },
    IncreaseColourTemperature {
        value: u64,
    },
    DecreaseColourTemperature {
        value: u64,
    },
    StopColourTemperature,
    StopBrightness,
}

impl LightState {
    pub fn required_capability(&self) -> Option<Capability> {
        match self {
            LightState::On | LightState::Off | LightState::Toggle => None,
            LightState::SetBrightness { .. }
            | LightState::IncreaseBrightness { .. }
            | LightState::DecreaseBrightness { .. }
            | LightState::StopBrightness => Some(Capability::Brightness),
            LightState::IncreaseColourTemperature { .. }
            | LightState::DecreaseColourTemperature { .. }
            | LightState::StopColourTemperature => Some(Capability::ColourTemp),
        }
    }
}
