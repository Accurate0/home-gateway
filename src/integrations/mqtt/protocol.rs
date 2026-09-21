use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::decoding::DeviceRoleName;
use crate::integrations::esphome::EsphomeDomain;

use super::topic_template::TopicVars;

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

    pub fn roles(self) -> &'static [DeviceRoleName] {
        use DeviceRoleName::*;

        match self {
            MqttProtocol::Zigbee => &[
                Battery,
                Door,
                Environment,
                Light,
                SmartSwitch,
                Presence,
                ControlSwitch,
            ],
            MqttProtocol::Esphome => &[Environment, Plant, Light, Presence],
            MqttProtocol::Valetudo => &[RobotVacuum, Battery],
        }
    }

    pub fn supports(self, role: DeviceRoleName) -> bool {
        self.roles().contains(&role)
    }

    pub fn takes_entities(self) -> bool {
        match self {
            MqttProtocol::Esphome => true,
            MqttProtocol::Zigbee | MqttProtocol::Valetudo => false,
        }
    }

    pub fn parse_payload(self, vars: &TopicVars, payload: &[u8]) -> Option<Value> {
        match self {
            MqttProtocol::Esphome => {
                let domain = vars.get("domain")?;
                let domain: EsphomeDomain =
                    serde_json::from_value(Value::String(domain.clone())).ok()?;

                domain.parse(payload)
            }
            MqttProtocol::Zigbee | MqttProtocol::Valetudo => {
                match serde_json::from_slice::<Value>(payload) {
                    Ok(value) => Some(value),
                    Err(_) => std::str::from_utf8(payload)
                        .ok()
                        .map(|text| Value::String(text.to_owned())),
                }
            }
        }
    }

    pub fn payload_address(self, payload: &Value) -> Option<&str> {
        match self {
            MqttProtocol::Zigbee => payload.get("device")?.get("ieee_addr")?.as_str(),
            MqttProtocol::Esphome | MqttProtocol::Valetudo => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> TopicVars {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn esphome_payloads_are_parsed_by_their_topic_domain() {
        let protocol = MqttProtocol::Esphome;

        assert_eq!(
            protocol.parse_payload(&vars(&[("domain", "binary_sensor")]), b"ON"),
            Some(json!(true))
        );
        assert_eq!(
            protocol.parse_payload(&vars(&[("domain", "sensor")]), b"21.5"),
            Some(json!(21.5))
        );
        assert_eq!(
            protocol.parse_payload(&vars(&[("domain", "fan")]), b"ON"),
            None
        );
    }

    #[test]
    fn other_payloads_are_json_or_text() {
        let protocol = MqttProtocol::Valetudo;

        assert_eq!(
            protocol.parse_payload(&TopicVars::new(), br#"{"state":"docked"}"#),
            Some(json!({ "state": "docked" }))
        );
        assert_eq!(
            protocol.parse_payload(&TopicVars::new(), b"docked"),
            Some(json!("docked"))
        );
    }

    #[test]
    fn zigbee_payloads_name_their_ieee_address() {
        let payload = json!({ "device": { "ieee_addr": "0xabc" } });

        assert_eq!(
            MqttProtocol::Zigbee.payload_address(&payload),
            Some("0xabc")
        );
        assert_eq!(MqttProtocol::Valetudo.payload_address(&payload), None);
    }
}
