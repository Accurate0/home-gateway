use crate::device_registry::DeviceRegistry;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::mqtt::MqttClient;
use crate::state::HandleRegistry;

pub struct CommandTargets<'a> {
    pub devices: &'a DeviceRegistry,
    pub mqtt: &'a MqttClient,
    pub home_assistant: Option<&'a HomeAssistant>,
}

impl<'a> CommandTargets<'a> {
    pub fn new(devices: &'a DeviceRegistry, handles: &'a HandleRegistry) -> Self {
        Self {
            devices,
            mqtt: handles.expect::<MqttClient>(),
            home_assistant: handles.get::<HomeAssistant>(),
        }
    }
}
