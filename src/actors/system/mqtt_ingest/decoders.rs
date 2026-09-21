use crate::decoding::ModelSources;
use crate::device_registry::Transport;
use crate::lua::{LuaDecoder, LuaError};
use crate::settings::LuaSettings;

pub struct Decoders {
    pub zigbee: LuaDecoder,
    pub esphome: LuaDecoder,
}

impl Decoders {
    pub fn load(sources: &ModelSources, settings: &LuaSettings) -> Result<Self, LuaError> {
        Ok(Self {
            zigbee: LuaDecoder::load(&Transport::Zigbee.to_string(), &sources.zigbee, settings)?,
            esphome: LuaDecoder::load(&Transport::Esphome.to_string(), &sources.esphome, settings)?,
        })
    }
}
