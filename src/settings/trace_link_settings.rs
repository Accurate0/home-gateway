use reqwest::Url;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value};

const TRACE_ID: &str = "{trace_id}";

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TraceLinkSettings {
    pub url: String,
    pub query: Map<String, Value>,
}

impl TraceLinkSettings {
    pub fn validate(&self) -> Result<(), String> {
        Url::parse(&self.url)
            .map_err(|e| format!("tracing.trace_link.url `{}` is not a url: {e}", self.url))?;

        if !self.query.values().any(mentions_trace_id) {
            return Err(format!(
                "tracing.trace_link.query must use `{TRACE_ID}` somewhere"
            ));
        }

        Ok(())
    }

    pub fn url_for(&self, trace_id: &str) -> Option<String> {
        let mut url = Url::parse(&self.url)
            .inspect_err(|e| tracing::warn!("trace link url `{}` is not a url: {e}", self.url))
            .ok()?;

        {
            let mut pairs = url.query_pairs_mut();

            for (name, value) in &self.query {
                pairs.append_pair(name, &parameter(&with_trace_id(value, trace_id)));
            }
        }

        Some(url.into())
    }
}

fn mentions_trace_id(value: &Value) -> bool {
    match value {
        Value::String(text) => text.contains(TRACE_ID),
        Value::Array(items) => items.iter().any(mentions_trace_id),
        Value::Object(fields) => fields.values().any(mentions_trace_id),
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}

fn with_trace_id(value: &Value, trace_id: &str) -> Value {
    match value {
        Value::String(text) => Value::String(text.replace(TRACE_ID, trace_id)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| with_trace_id(item, trace_id))
                .collect(),
        ),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(name, field)| (name.clone(), with_trace_id(field, trace_id)))
                .collect(),
        ),
        Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
    }
}

fn parameter(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Array(_) | Value::Object(_) => {
            value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    const TRACE: &str = "dba919365053a45dea0fc1e68d3672dc";

    fn link(yaml: &str) -> TraceLinkSettings {
        serde_yaml::from_str(yaml).expect("expected the trace link to parse")
    }

    fn grafana() -> TraceLinkSettings {
        link(
            r#"
url: https://grafana.example/explore
query:
  schemaVersion: 1
  orgId: 1
  panes:
    epd:
      datasource: tempo
      queries:
        - refId: A
          queryType: traceql
          query: "{trace_id}"
      range: { from: now-7d, to: now }
"#,
        )
    }

    fn parameters(link: &str) -> Vec<(String, String)> {
        Url::parse(link)
            .unwrap()
            .query_pairs()
            .map(|(name, value)| (name.into_owned(), value.into_owned()))
            .collect()
    }

    #[test]
    fn the_link_carries_every_parameter_with_the_trace_id_filled_in() {
        let url = grafana().url_for(TRACE).unwrap();
        let parameters = parameters(&url);

        assert!(url.starts_with("https://grafana.example/explore?"));
        assert_eq!(parameters[0], ("schemaVersion".to_owned(), "1".to_owned()));
        assert_eq!(parameters[1], ("orgId".to_owned(), "1".to_owned()));
        assert_eq!(parameters[2].0, "panes");

        let panes: Value = serde_json::from_str(&parameters[2].1).unwrap();

        assert_eq!(
            panes,
            json!({
                "epd": {
                    "datasource": "tempo",
                    "queries": [{ "refId": "A", "queryType": "traceql", "query": TRACE }],
                    "range": { "from": "now-7d", "to": "now" }
                }
            })
        );
    }

    #[test]
    fn structured_parameters_are_percent_encoded_in_the_link() {
        let url = grafana().url_for(TRACE).unwrap();

        assert!(!url.contains('{'));
        assert!(!url.contains('"'));
        assert!(url.contains(TRACE));
    }

    #[test]
    fn a_plain_string_parameter_is_not_quoted() {
        let link = link("url: https://tempo.example/trace\nquery: { id: \"{trace_id}\" }");

        assert_eq!(
            link.url_for(TRACE).unwrap(),
            format!("https://tempo.example/trace?id={TRACE}")
        );
    }

    #[test]
    fn a_query_without_the_trace_id_is_rejected() {
        let link = link("url: https://grafana.example/explore\nquery: { orgId: 1 }");

        assert!(link.validate().unwrap_err().contains("{trace_id}"));
    }

    #[test]
    fn a_malformed_url_is_rejected() {
        let link = link("url: grafana\nquery: { id: \"{trace_id}\" }");

        assert!(link.validate().unwrap_err().contains("is not a url"));
        assert!(grafana().validate().is_ok());
    }
}
