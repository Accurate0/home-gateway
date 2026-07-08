use async_graphql::{InputObject, Object};
use ractor::factory::{FactoryMessage, Job, JobOptions};

use crate::actors::light::{LightHandler, LightHandlerMessage};
use crate::auth::scope::required;
use crate::device_registry::Capability;
use crate::graphql::guard::ScopeGuard;
use crate::settings::IEEEAddress;

pub struct LightMutation {
    pub address: IEEEAddress,
    pub capabilities: Vec<Capability>,
}

#[derive(InputObject)]
pub struct SetBrightnessInput {
    pub value: u64,
}

#[derive(InputObject)]
pub struct BrightnessMoveInput {
    pub value: i64,
    pub on_off: bool,
}

#[derive(InputObject)]
pub struct ColourTemperatureMoveInput {
    pub value: i64,
}

#[derive(InputObject)]
pub struct SetColourInput {
    pub hex: String,
}

fn is_valid_hex(hex: &str) -> bool {
    hex.len() == 7
        && hex.starts_with('#')
        && hex[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

fn dispatch(message: LightHandlerMessage) -> async_graphql::Result<bool> {
    let Some(actor) = ractor::registry::where_is(LightHandler::NAME.to_string()) else {
        return Err(async_graphql::Error::new("light actor unavailable"));
    };

    let message = FactoryMessage::Dispatch(Job {
        key: (),
        msg: message,
        options: JobOptions::default(),
        accepted: None,
    });

    actor.send_message(message)?;

    Ok(true)
}

#[Object]
impl LightMutation {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_LIGHT_WRITE))]
    async fn on(&self) -> async_graphql::Result<bool> {
        dispatch(LightHandlerMessage::TurnOn {
            ieee_addr: self.address.clone(),
        })
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_LIGHT_WRITE))]
    async fn off(&self) -> async_graphql::Result<bool> {
        dispatch(LightHandlerMessage::TurnOff {
            ieee_addr: self.address.clone(),
        })
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_LIGHT_WRITE))]
    async fn toggle(&self) -> async_graphql::Result<bool> {
        dispatch(LightHandlerMessage::Toggle {
            ieee_addr: self.address.clone(),
        })
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_LIGHT_WRITE))]
    async fn set_brightness(&self, input: SetBrightnessInput) -> async_graphql::Result<bool> {
        dispatch(LightHandlerMessage::SetBrightness {
            ieee_addr: self.address.clone(),
            value: input.value,
        })
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_LIGHT_WRITE))]
    async fn brightness_move(&self, input: BrightnessMoveInput) -> async_graphql::Result<bool> {
        dispatch(LightHandlerMessage::BrightnessMove {
            ieee_addr: self.address.clone(),
            value: input.value,
            on_off: input.on_off,
        })
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_LIGHT_WRITE))]
    async fn set_colour(&self, input: SetColourInput) -> async_graphql::Result<bool> {
        if !self.capabilities.contains(&Capability::Rgb) {
            return Err(async_graphql::Error::new("light does not support RGB colour"));
        }
        if !is_valid_hex(&input.hex) {
            return Err(async_graphql::Error::new("invalid hex colour, expected #RRGGBB"));
        }
        dispatch(LightHandlerMessage::SetColour {
            ieee_addr: self.address.clone(),
            hex: input.hex,
        })
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_LIGHT_WRITE))]
    async fn colour_temperature_move(
        &self,
        input: ColourTemperatureMoveInput,
    ) -> async_graphql::Result<bool> {
        dispatch(LightHandlerMessage::ColourTemperatureMove {
            ieee_addr: self.address.clone(),
            value: input.value,
        })
    }
}
