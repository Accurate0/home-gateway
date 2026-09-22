use serde::Serialize;

use crate::repo::light::LightAttributes;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LightCommand {
    Set {
        on: Option<bool>,
        brightness: Option<u64>,
        colour_temp: Option<u64>,
        colour: Option<String>,
    },
    Toggle,
    BrightnessMove {
        value: i64,
        on_off: bool,
    },
    ColourTempMove {
        value: i64,
    },
}

impl LightCommand {
    pub fn power(on: bool) -> Self {
        LightCommand::Set {
            on: Some(on),
            brightness: None,
            colour_temp: None,
            colour: None,
        }
    }

    pub fn reapply(attributes: &LightAttributes) -> Self {
        LightCommand::Set {
            on: attributes.state.as_deref().map(|state| state == "ON"),
            brightness: attributes
                .brightness
                .and_then(|value| value.try_into().ok()),
            colour_temp: attributes
                .colour_temp
                .and_then(|value| value.try_into().ok()),
            colour: attributes.colour.clone(),
        }
    }
}
