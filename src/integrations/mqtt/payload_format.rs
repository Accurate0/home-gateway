use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::integrations::esphome::EsphomeDomain;

use super::topic_template::TopicVars;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PayloadFormat {
    Json,
    EsphomeDomain,
}

impl std::fmt::Display for PayloadFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            PayloadFormat::Json => "json",
            PayloadFormat::EsphomeDomain => "esphome_domain",
        };

        f.write_str(name)
    }
}

impl PayloadFormat {
    pub fn parse(self, vars: &TopicVars, payload: &[u8]) -> Option<Value> {
        match self {
            PayloadFormat::EsphomeDomain => {
                let domain = vars.get("domain")?;
                let domain: EsphomeDomain =
                    serde_json::from_value(Value::String(domain.clone())).ok()?;

                domain.parse(payload)
            }
            PayloadFormat::Json => match serde_json::from_slice::<Value>(payload) {
                Ok(value) => Some(value),
                Err(_) => std::str::from_utf8(payload)
                    .ok()
                    .map(|text| Value::String(text.to_owned())),
            },
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
        let format = PayloadFormat::EsphomeDomain;

        assert_eq!(
            format.parse(&vars(&[("domain", "binary_sensor")]), b"ON"),
            Some(json!(true))
        );
        assert_eq!(
            format.parse(&vars(&[("domain", "sensor")]), b"21.5"),
            Some(json!(21.5))
        );
        assert_eq!(format.parse(&vars(&[("domain", "fan")]), b"ON"), None);
    }

    #[test]
    fn json_payloads_fall_back_to_text() {
        let format = PayloadFormat::Json;

        assert_eq!(
            format.parse(&TopicVars::new(), br#"{"state":"docked"}"#),
            Some(json!({ "state": "docked" }))
        );
        assert_eq!(
            format.parse(&TopicVars::new(), b"docked"),
            Some(json!("docked"))
        );
    }
}
