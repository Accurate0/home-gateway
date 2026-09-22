use crate::decoding::DeviceRoleName;
use crate::device_registry::Transport;
use crate::integrations::home_assistant::HomeAssistantError;
use crate::integrations::mqtt::MqttError;

#[derive(Debug, thiserror::Error)]
pub enum DeviceCommandError {
    #[error("{0} is not a registered device")]
    UnknownDevice(String),
    #[error("{address}: a `{transport}` device can't take `{role}` commands")]
    Unsupported {
        address: String,
        transport: Transport,
        role: DeviceRoleName,
    },
    #[error("{0}: home assistant commands are `domain.service` names, not payloads")]
    PayloadForService(String),
    #[error("home assistant is not configured")]
    HomeAssistantNotConfigured,
    #[error(transparent)]
    HomeAssistant(#[from] HomeAssistantError),
    #[error(transparent)]
    Mqtt(#[from] MqttError),
    #[error("no mqtt command topic: {0}")]
    CommandTopic(String),
}
