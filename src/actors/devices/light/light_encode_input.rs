use serde::Serialize;

use super::light_command::LightCommand;
use super::light_current::LightCurrent;

#[derive(Debug, Serialize)]
pub struct LightEncodeInput<'a> {
    pub command: &'a LightCommand,
    pub current: LightCurrent<'a>,
}
