use std::num::TryFromIntError;

use crate::settings::IEEEAddress;
use crate::settings::workflow::LightState;

use super::LightHandlerMessage;

pub fn light_message(
    ieee_addr: IEEEAddress,
    state: LightState,
) -> Result<LightHandlerMessage, TryFromIntError> {
    let message = match state {
        LightState::On => LightHandlerMessage::TurnOn { ieee_addr },
        LightState::Off => LightHandlerMessage::TurnOff { ieee_addr },
        LightState::Toggle => LightHandlerMessage::Toggle { ieee_addr },
        LightState::SetBrightness { value } => {
            LightHandlerMessage::SetBrightness { ieee_addr, value }
        }
        LightState::IncreaseBrightness { value, on_off } => LightHandlerMessage::BrightnessMove {
            ieee_addr,
            value: value.try_into()?,
            on_off,
        },
        LightState::DecreaseBrightness { value, on_off } => LightHandlerMessage::BrightnessMove {
            ieee_addr,
            value: -TryInto::<i64>::try_into(value)?,
            on_off,
        },
        LightState::StopBrightness => LightHandlerMessage::BrightnessMove {
            ieee_addr,
            value: 0,
            on_off: false,
        },
        LightState::IncreaseColourTemperature { value } => {
            LightHandlerMessage::ColourTemperatureMove {
                ieee_addr,
                value: value.try_into()?,
            }
        }
        LightState::DecreaseColourTemperature { value } => {
            LightHandlerMessage::ColourTemperatureMove {
                ieee_addr,
                value: -TryInto::<i64>::try_into(value)?,
            }
        }
        LightState::StopColourTemperature => LightHandlerMessage::ColourTemperatureMove {
            ieee_addr,
            value: 0,
        },
    };

    Ok(message)
}
