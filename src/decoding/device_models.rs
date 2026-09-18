use super::models::Models;

#[derive(Debug, Default)]
pub struct DeviceModels {
    pub zigbee: Models,
    pub home_assistant: Models,
}
