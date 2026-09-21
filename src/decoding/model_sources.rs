use std::collections::BTreeMap;

use crate::device_registry::Transport;
use crate::settings::LuaSettings;

use super::device_models::DeviceModels;
use super::models::load_models;

#[derive(Debug, Default, Clone)]
pub struct ModelSources {
    pub zigbee: BTreeMap<String, String>,
    pub esphome: BTreeMap<String, String>,
    pub home_assistant: BTreeMap<String, String>,
}

impl ModelSources {
    pub fn load(&self, settings: &LuaSettings) -> Result<DeviceModels, String> {
        Ok(DeviceModels {
            zigbee: load_models(Transport::Zigbee, &self.zigbee, settings)?,
            esphome: load_models(Transport::Esphome, &self.esphome, settings)?,
            home_assistant: load_models(Transport::HomeAssistant, &self.home_assistant, settings)?,
        })
    }
}
