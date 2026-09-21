use std::collections::BTreeMap;

use crate::device_registry::Transport;
use crate::settings::LuaSettings;

use super::device_models::DeviceModels;
use super::models::load_models;

#[derive(Debug, Default, Clone)]
pub struct ModelSources {
    pub mqtt: BTreeMap<String, String>,
    pub home_assistant: BTreeMap<String, String>,
}

impl ModelSources {
    pub fn load(&self, settings: &LuaSettings) -> Result<DeviceModels, String> {
        Ok(DeviceModels {
            mqtt: load_models(Transport::Mqtt, &self.mqtt, settings)?,
            home_assistant: load_models(Transport::HomeAssistant, &self.home_assistant, settings)?,
        })
    }
}
