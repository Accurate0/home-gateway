use std::collections::BTreeMap;

use crate::device_registry::Transport;
use crate::lua::{LuaDecoder, LuaError};
use crate::settings::{LuaSettings, MqttProtocols};

use super::device_models::DeviceModels;
use super::models::load_models;

#[derive(Debug, Default, Clone)]
pub struct ModelSources {
    pub mqtt: BTreeMap<String, String>,
    pub esphome_native_api: BTreeMap<String, String>,
    pub home_assistant: BTreeMap<String, String>,
    pub library: BTreeMap<String, String>,
}

impl ModelSources {
    pub fn load(
        &self,
        settings: &LuaSettings,
        protocols: &MqttProtocols,
    ) -> Result<DeviceModels, String> {
        Ok(DeviceModels {
            mqtt: load_models(
                Transport::Mqtt,
                &self.mqtt,
                &self.library,
                settings,
                protocols,
            )?,
            esphome_native_api: load_models(
                Transport::EsphomeNativeApi,
                &self.esphome_native_api,
                &self.library,
                settings,
                protocols,
            )?,
            home_assistant: load_models(
                Transport::HomeAssistant,
                &self.home_assistant,
                &self.library,
                settings,
                protocols,
            )?,
        })
    }

    pub fn decoder(
        &self,
        transport: Transport,
        settings: &LuaSettings,
    ) -> Result<LuaDecoder, LuaError> {
        let sources = match transport {
            Transport::Mqtt => &self.mqtt,
            Transport::EsphomeNativeApi => &self.esphome_native_api,
            Transport::HomeAssistant => &self.home_assistant,
            Transport::EinkDisplayFirmware | Transport::Trmnl => {
                return Err(LuaError::Runtime(format!(
                    "{transport} devices have no lua models"
                )));
            }
        };

        LuaDecoder::load(&transport.to_string(), sources, &self.library, settings)
    }
}
