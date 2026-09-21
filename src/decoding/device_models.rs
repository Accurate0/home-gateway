use crate::device_registry::Transport;

use super::models::Models;

#[derive(Debug, Default)]
pub struct DeviceModels {
    pub zigbee: Models,
    pub esphome: Models,
    pub home_assistant: Models,
}

impl DeviceModels {
    pub fn for_transport(&self, transport: Transport) -> Option<&Models> {
        match transport {
            Transport::Zigbee => Some(&self.zigbee),
            Transport::Esphome => Some(&self.esphome),
            Transport::HomeAssistant => Some(&self.home_assistant),
            Transport::EinkDisplayFirmware | Transport::Trmnl | Transport::Valetudo => None,
        }
    }
}
