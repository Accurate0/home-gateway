use serde::Serialize;

use crate::repo::light::LightState;

#[derive(Debug, Serialize)]
pub struct LightCurrent<'a> {
    pub on: bool,
    pub brightness: Option<i32>,
    pub colour_temp: Option<i32>,
    pub colour: Option<&'a str>,
}

impl<'a> From<&'a LightState> for LightCurrent<'a> {
    fn from(state: &'a LightState) -> Self {
        Self {
            on: state.on,
            brightness: state.brightness,
            colour_temp: state.colour_temp,
            colour: state.colour.as_deref(),
        }
    }
}
