use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::integrations::mqtt::{MqttProtocol, TopicTemplate};

pub const COMMAND_VARS: [&str; 4] = ["address", "name", "domain", "object_id"];

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MqttProtocolSettings {
    pub topics: BTreeMap<String, TopicTemplate>,
    pub command: Option<TopicTemplate>,
    pub directory: Option<TopicTemplate>,
    pub discovery: Option<TopicTemplate>,
}

impl MqttProtocolSettings {
    pub fn validate(&self, protocol: MqttProtocol) -> Result<(), String> {
        let identifies =
            |template: &TopicTemplate| template.has_var("address") || template.has_var("name");

        if self.topics.is_empty() {
            return Err(format!("mqtt.protocols.{protocol}.topics is empty"));
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

    fn validate(yaml: &str) -> Result<(), String> {
        serde_yaml::from_str::<MqttProtocolSettings>(yaml)
            .unwrap()
            .validate(MqttProtocol::Valetudo)
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
