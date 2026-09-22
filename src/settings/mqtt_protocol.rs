use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::decoding::DeviceRoleName;
use crate::device_registry::Transport;
use crate::integrations::mqtt::{MqttProtocol, PayloadFormat, TopicTemplate, TopicVars};

use super::enabled_state::EnabledState;

pub const COMMAND_VARS: [&str; 4] = ["address", "name", "domain", "object_id"];

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MqttProtocolSettings {
    pub payload: PayloadFormat,
    pub address_path: Option<Vec<String>>,
    pub entities: EnabledState,
    pub roles: BTreeSet<DeviceRoleName>,
    pub topics: BTreeMap<String, TopicTemplate>,
    pub command: Option<TopicTemplate>,
    pub directory: Option<TopicTemplate>,
    pub discovery: Option<TopicTemplate>,
}

impl MqttProtocolSettings {
    pub fn supports(&self, role: DeviceRoleName) -> bool {
        self.roles.contains(&role)
    }

    pub fn takes_entities(&self) -> bool {
        self.entities.is_enabled()
    }

    pub fn parse_payload(&self, vars: &TopicVars, payload: &[u8]) -> Option<Value> {
        self.payload.parse(vars, payload)
    }

    pub fn payload_address<'a>(&self, payload: &'a Value) -> Option<&'a str> {
        self.address_path
            .as_ref()?
            .iter()
            .try_fold(payload, |value, key| value.get(key))?
            .as_str()
    }

    pub fn validate(&self, protocol: MqttProtocol) -> Result<(), String> {
        let identifies =
            |template: &TopicTemplate| template.has_var("address") || template.has_var("name");

        if self.topics.is_empty() {
            return Err(format!("mqtt.protocols.{protocol}.topics is empty"));
        }

        if self.roles.is_empty() {
            return Err(format!("mqtt.protocols.{protocol}.roles is empty"));
        }

        if let Some(role) = self
            .roles
            .iter()
            .find(|role| !Transport::Mqtt.supports(**role))
        {
            return Err(format!(
                "mqtt.protocols.{protocol}.roles lists `{role}`, which no mqtt device can carry"
            ));
        }

        if self.address_path.as_ref().is_some_and(Vec::is_empty) {
            return Err(format!("mqtt.protocols.{protocol}.address_path is empty"));
        }

        if self.payload == PayloadFormat::EsphomeDomain
            && !self
                .topics
                .values()
                .any(|template| template.has_var("domain"))
        {
            return Err(format!(
                "mqtt.protocols.{protocol}: the `{}` payload needs a topic with `{{domain}}`",
                self.payload
            ));
        }

        for (name, template) in &self.topics {
            if !identifies(template) {
                return Err(format!(
                    "mqtt.protocols.{protocol}.topics.{name} `{template}` must contain `{{address}}` or `{{name}}`"
                ));
            }
        }

        if let Some(command) = &self.command {
            if !identifies(command) {
                return Err(format!(
                    "mqtt.protocols.{protocol}.command `{command}` must contain `{{address}}` or `{{name}}`"
                ));
            }

            if let Some(var) = command.vars().find(|var| !COMMAND_VARS.contains(var)) {
                return Err(format!(
                    "mqtt.protocols.{protocol}.command `{command}` uses unknown `{{{var}}}`; commands can use {}",
                    COMMAND_VARS.join(", ")
                ));
            }
        }

        if let Some(directory) = &self.directory
            && directory.vars().next().is_some()
        {
            return Err(format!(
                "mqtt.protocols.{protocol}.directory `{directory}` must be a fixed topic"
            ));
        }

        if let Some(discovery) = &self.discovery
            && !discovery.has_var("address")
        {
            return Err(format!(
                "mqtt.protocols.{protocol}.discovery `{discovery}` must contain `{{address}}`"
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQUIRED: &str = "
payload: json
entities: disabled
roles: [robot_vacuum, battery]
";

    fn validate(yaml: &str) -> Result<(), String> {
        serde_yaml::from_str::<MqttProtocolSettings>(&format!("{REQUIRED}{yaml}"))
            .unwrap()
            .validate(MqttProtocol::Valetudo)
    }

    #[test]
    fn roles_must_be_ones_an_mqtt_device_can_carry() {
        let settings: MqttProtocolSettings = serde_yaml::from_str(
            r#"
payload: json
entities: disabled
roles: [trmnl]
topics: { state: "valetudo/{address}/state" }
"#,
        )
        .unwrap();

        let error = settings.validate(MqttProtocol::Valetudo).unwrap_err();

        assert!(error.contains("`trmnl`"), "{error}");
    }

    #[test]
    fn the_esphome_payload_needs_a_domain_in_its_topics() {
        let settings: MqttProtocolSettings = serde_yaml::from_str(
            r#"
payload: esphome_domain
entities: enabled
roles: [environment]
topics: { state: "{address}/state" }
"#,
        )
        .unwrap();

        let error = settings.validate(MqttProtocol::Esphome).unwrap_err();

        assert!(error.contains("`{domain}`"), "{error}");
    }

    #[test]
    fn the_address_path_finds_the_device_in_a_payload() {
        let with_path: MqttProtocolSettings = serde_yaml::from_str(&format!(
            "{REQUIRED}address_path: [device, ieee_addr]\ntopics: {{ report: \"z/{{name}}\" }}"
        ))
        .unwrap();
        let without: MqttProtocolSettings =
            serde_yaml::from_str(&format!("{REQUIRED}topics: {{ report: \"z/{{name}}\" }}"))
                .unwrap();

        let payload = serde_json::json!({ "device": { "ieee_addr": "0xabc" } });

        assert_eq!(with_path.payload_address(&payload), Some("0xabc"));
        assert_eq!(without.payload_address(&payload), None);
    }

    #[test]
    fn a_complete_protocol_validates() {
        validate(
            r#"
topics: { state: "valetudo/{address}/state" }
command: "valetudo/{address}/command"
"#,
        )
        .unwrap();
    }

    #[test]
    fn every_topic_must_identify_its_device() {
        let error = validate(r#"topics: { state: "valetudo/{leaf}" }"#).unwrap_err();

        assert!(
            error.contains("must contain `{address}` or `{name}`"),
            "{error}"
        );
    }

    #[test]
    fn a_protocol_needs_at_least_one_topic() {
        let error = validate("topics: {}").unwrap_err();

        assert!(error.contains("topics is empty"), "{error}");
    }

    #[test]
    fn commands_only_use_vars_the_gateway_can_fill() {
        let error = validate(
            r#"
topics: { state: "valetudo/{address}/state" }
command: "valetudo/{address}/{segment}/command"
"#,
        )
        .unwrap_err();

        assert!(error.contains("unknown `{segment}`"), "{error}");
    }

    #[test]
    fn a_directory_is_a_fixed_topic_and_discovery_names_an_address() {
        let directory = validate(
            r#"
topics: { state: "valetudo/{address}/state" }
directory: "valetudo/{address}/devices"
"#,
        )
        .unwrap_err();
        assert!(directory.contains("must be a fixed topic"), "{directory}");

        let discovery = validate(
            r#"
topics: { state: "valetudo/{address}/state" }
discovery: "valetudo/discover/{name}"
"#,
        )
        .unwrap_err();
        assert!(
            discovery.contains("must contain `{address}`"),
            "{discovery}"
        );
    }
}
