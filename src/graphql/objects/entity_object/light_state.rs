use async_graphql::SimpleObject;

use crate::repo::light::LightState;

#[derive(SimpleObject)]
#[graphql(name = "LightState")]
pub struct LightStateObject {
    pub on: bool,
    /// Current brightness on the light model's `ranges.brightness` scale, when the light reports one.
    pub brightness: Option<i32>,
    /// Colour temperature in mireds (1000000/kelvin), within the light model's `ranges.colour_temp`.
    pub colour_temperature: Option<i32>,
    /// Current colour as `#rrggbb`, when the light reports one.
    pub colour: Option<String>,
}

impl From<LightState> for LightStateObject {
    fn from(state: LightState) -> Self {
        Self {
            on: state.on,
            brightness: state.brightness,
            colour_temperature: state.colour_temp,
            colour: state.colour,
        }
    }
}
