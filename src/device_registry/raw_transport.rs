use schemars::JsonSchema;
use serde::Deserialize;

use super::transport::Transport;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RawTransport {
    Mqtt { address: String },
    EsphomeNativeApi { address: String },
    EinkDisplayFirmware { address: String },
    Trmnl { address: String },
    HomeAssistant { address: String },
}

impl RawTransport {
    pub fn kind(&self) -> Transport {
        match self {
            RawTransport::Mqtt { .. } => Transport::Mqtt,
            RawTransport::EsphomeNativeApi { .. } => Transport::EsphomeNativeApi,
            RawTransport::EinkDisplayFirmware { .. } => Transport::EinkDisplayFirmware,
            RawTransport::Trmnl { .. } => Transport::Trmnl,
            RawTransport::HomeAssistant { .. } => Transport::HomeAssistant,
        }
    }

    pub fn into_address(self) -> String {
        match self {
            RawTransport::Mqtt { address }
            | RawTransport::EsphomeNativeApi { address }
            | RawTransport::EinkDisplayFirmware { address }
            | RawTransport::Trmnl { address }
            | RawTransport::HomeAssistant { address } => address,
        }
    }
}
