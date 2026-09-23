use crate::device_registry::Transport;
use crate::integrations::esphome_native_api::EsphomeNativeApiError;
use crate::integrations::home_assistant::HomeAssistantError;

#[derive(Debug, thiserror::Error)]
pub enum MediaControlError {
    #[error("{0} is not a registered device")]
    UnknownDevice(String),
    #[error("{address}: a `{transport}` device can't take media commands")]
    Unsupported {
        address: String,
        transport: Transport,
    },
    #[error("home assistant is not configured")]
    HomeAssistantNotConfigured,
    #[error("the esphome native api is not configured")]
    EsphomeNativeApiNotConfigured,
    #[error(transparent)]
    HomeAssistant(#[from] HomeAssistantError),
    #[error(transparent)]
    EsphomeNativeApi(#[from] EsphomeNativeApiError),
}
