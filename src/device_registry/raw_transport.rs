use schemars::JsonSchema;
use serde::Deserialize;

use super::transport::Transport;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RawTransport {
    Zigbee { address: String },
    Esphome { address: String },
    EinkDisplayFirmware { address: String },
    Trmnl { address: String },
    HomeAssistant { address: String },
    Valetudo { address: String },
}

impl RawTransport {
    pub fn kind(&self) -> Transport {
        match self {
            RawTransport::Zigbee { .. } => Transport::Zigbee,
            RawTransport::Esphome { .. } => Transport::Esphome,
            RawTransport::EinkDisplayFirmware { .. } => Transport::EinkDisplayFirmware,
            RawTransport::Trmnl { .. } => Transport::Trmnl,
            RawTransport::HomeAssistant { .. } => Transport::HomeAssistant,
            RawTransport::Valetudo { .. } => Transport::Valetudo,
        }
    }

    pub fn into_address(self) -> String {
        match self {
            RawTransport::Zigbee { address }
            | RawTransport::Esphome { address }
            | RawTransport::EinkDisplayFirmware { address }
            | RawTransport::Trmnl { address }
            | RawTransport::HomeAssistant { address }
            | RawTransport::Valetudo { address } => address,
        }
    }
}
