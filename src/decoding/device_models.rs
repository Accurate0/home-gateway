use crate::device_registry::Transport;

use super::models::Models;

#[derive(Debug, Default)]
pub struct DeviceModels {
    pub mqtt: Models,
    pub home_assistant: Models,
}

impl DeviceModels {
    pub fn for_transport(&self, transport: Transport) -> Option<&Models> {
        match transport {
            Transport::Mqtt => Some(&self.mqtt),
            Transport::HomeAssistant => Some(&self.home_assistant),
            Transport::EinkDisplayFirmware | Transport::Trmnl => None,
        }
    }
}
