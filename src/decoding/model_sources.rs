use std::collections::BTreeMap;

use crate::settings::LuaSettings;

use super::device_models::DeviceModels;
use super::models::load_models;

#[derive(Debug, Default, Clone)]
pub struct ModelSources {
    pub zigbee: BTreeMap<String, String>,
    pub home_assistant: BTreeMap<String, String>,
}

impl ModelSources {
    pub fn load(&self, settings: &LuaSettings) -> Result<DeviceModels, String> {
        Ok(DeviceModels {
            zigbee: load_models("zigbee", &self.zigbee, settings)?,
            home_assistant: load_models("home_assistant", &self.home_assistant, settings)?,
        })
    }
}
