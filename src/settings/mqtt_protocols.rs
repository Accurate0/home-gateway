use schemars::JsonSchema;
use serde::Deserialize;

use crate::integrations::mqtt::MqttProtocol;

use super::mqtt_protocol::MqttProtocolSettings;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MqttProtocols {
    pub zigbee: MqttProtocolSettings,
    pub esphome: MqttProtocolSettings,
    pub valetudo: MqttProtocolSettings,
}

impl MqttProtocols {
    pub fn get(&self, protocol: MqttProtocol) -> &MqttProtocolSettings {
        match protocol {
            MqttProtocol::Zigbee => &self.zigbee,
            MqttProtocol::Esphome => &self.esphome,
            MqttProtocol::Valetudo => &self.valetudo,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (MqttProtocol, &MqttProtocolSettings)> {
        MqttProtocol::ALL
            .into_iter()
            .map(|protocol| (protocol, self.get(protocol)))
    }

    pub fn validate(&self) -> Result<(), String> {
        self.iter()
            .try_for_each(|(protocol, settings)| settings.validate(protocol))
    }

    #[cfg(test)]
    pub fn committed() -> Self {
        let section: serde_yaml::Value =
            serde_yaml::from_str(include_str!("../../config/sections/mqtt.yaml")).unwrap();

        serde_yaml::from_value(section["protocols"].clone()).unwrap()
    }
}
