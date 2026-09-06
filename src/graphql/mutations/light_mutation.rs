use async_graphql::{InputObject, Object};

use std::time::Duration;

use crate::actors::devices::light::{LightHandler, LightHandlerMessage, SetRequest};
use crate::actors::system::rpc;
use crate::auth::scope::{Action, Resource, Scope};
use crate::device_registry::Capability;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::entity_object::LightStateObject;
use crate::settings::IEEEAddress;

const SET_TIMEOUT: Duration = Duration::from_secs(10);

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
pub struct SetColourTemperatureInput {
    /// Colour temperature in mireds (1000000/kelvin): 153 is coolest, 500 warmest.
    pub value: u64,
}

#[derive(InputObject)]
pub struct SetColourInput {
    pub hex: String,
}

#[derive(InputObject)]
pub struct LightSetInput {
    pub on: Option<bool>,
    pub brightness: Option<u64>,
    /// Colour temperature in mireds (1000000/kelvin): 153 is coolest, 500 warmest.
    pub colour_temperature: Option<u64>,
    pub colour: Option<String>,
}

impl LightMutation {
    fn require(&self, capability: Capability) -> async_graphql::Result<()> {
        if self.capabilities.contains(&capability) {
            Ok(())
        } else {
            Err(async_graphql::Error::new(format!(
                "light does not support {capability:?}"
            )))
        }
    }
}

fn is_valid_hex(hex: &str) -> bool {
    hex.len() == 7 && hex.starts_with('#') && hex[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

fn dispatch(message: LightHandlerMessage) -> async_graphql::Result<bool> {
    rpc::cast_factory(LightHandler::NAME, message)?;

    Ok(true)
}

#[Object]
impl LightMutation {
    /// Apply any combination of power, brightness, colour temperature and colour
    /// in one command, and return the light's resulting state.
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Write)))]
    async fn set(&self, input: LightSetInput) -> async_graphql::Result<LightStateObject> {
        if input.brightness.is_some() {
            self.require(Capability::Brightness)?;
        }

        if input.colour_temperature.is_some() {
            self.require(Capability::ColourTemp)?;
        }

        if let Some(colour) = &input.colour {
            self.require(Capability::Rgb)?;
            if !is_valid_hex(colour) {
                return Err(async_graphql::Error::new(
                    "invalid hex colour, expected #RRGGBB",
                ));
            }
        }

        let state = rpc::query_factory(LightHandler::NAME, SET_TIMEOUT, |reply| {
            LightHandlerMessage::Set {
                ieee_addr: self.address.clone(),
                request: Box::new(SetRequest {
                    on: input.on,
                    brightness: input.brightness,
                    colour_temp: input.colour_temperature,
                    colour: input.colour,
                }),
                reply,
            }
        })
        .await?;

        Ok(state.into())
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Write)))]
    async fn on(&self) -> async_graphql::Result<bool> {
        dispatch(LightHandlerMessage::TurnOn {
            ieee_addr: self.address.clone(),
        })
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Write)))]
    async fn off(&self) -> async_graphql::Result<bool> {
        dispatch(LightHandlerMessage::TurnOff {
            ieee_addr: self.address.clone(),
        })
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Write)))]
    async fn toggle(&self) -> async_graphql::Result<bool> {
        dispatch(LightHandlerMessage::Toggle {
            ieee_addr: self.address.clone(),
        })
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Write)))]
    async fn set_brightness(&self, input: SetBrightnessInput) -> async_graphql::Result<bool> {
        self.require(Capability::Brightness)?;
        dispatch(LightHandlerMessage::SetBrightness {
            ieee_addr: self.address.clone(),
            value: input.value,
        })
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Write)))]
    async fn brightness_move(&self, input: BrightnessMoveInput) -> async_graphql::Result<bool> {
        self.require(Capability::Brightness)?;
        dispatch(LightHandlerMessage::BrightnessMove {
            ieee_addr: self.address.clone(),
            value: input.value,
            on_off: input.on_off,
        })
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Write)))]
    async fn set_colour(&self, input: SetColourInput) -> async_graphql::Result<bool> {
        self.require(Capability::Rgb)?;
        if !is_valid_hex(&input.hex) {
            return Err(async_graphql::Error::new(
                "invalid hex colour, expected #RRGGBB",
            ));
        }
        dispatch(LightHandlerMessage::SetColour {
            ieee_addr: self.address.clone(),
            hex: input.hex,
        })
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Write)))]
    async fn set_colour_temperature(
        &self,
        input: SetColourTemperatureInput,
    ) -> async_graphql::Result<bool> {
        self.require(Capability::ColourTemp)?;
        dispatch(LightHandlerMessage::SetColourTemperature {
            ieee_addr: self.address.clone(),
            value: input.value,
        })
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Light, Action::Write)))]
    async fn colour_temperature_move(
        &self,
        input: ColourTemperatureMoveInput,
    ) -> async_graphql::Result<bool> {
        self.require(Capability::ColourTemp)?;
        dispatch(LightHandlerMessage::ColourTemperatureMove {
            ieee_addr: self.address.clone(),
            value: input.value,
        })
    }
}
