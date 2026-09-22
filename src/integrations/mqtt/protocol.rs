use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MqttProtocol {
    Zigbee,
    Esphome,
    Valetudo,
}

impl std::fmt::Display for MqttProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            MqttProtocol::Zigbee => "zigbee",
            MqttProtocol::Esphome => "esphome",
            MqttProtocol::Valetudo => "valetudo",
        };

        f.write_str(name)
    }
}

impl MqttProtocol {
    pub const ALL: [MqttProtocol; 3] = [
        MqttProtocol::Zigbee,
        MqttProtocol::Esphome,
        MqttProtocol::Valetudo,
    ];
}
