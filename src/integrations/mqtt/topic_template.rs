use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Literal(String),
    Var(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String")]
pub struct TopicTemplate {
    raw: String,
    segments: Vec<Segment>,
}

pub type TopicVars = BTreeMap<String, String>;

impl TryFrom<String> for TopicTemplate {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let mut segments = Vec::new();

        for segment in raw.split('/') {
            let parsed = match segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                Some(name) if !name.is_empty() => Segment::Var(name.to_owned()),
                Some(_) => return Err(format!("topic `{raw}` has an empty `{{}}` placeholder")),
                None if segment.contains(['{', '}', '+', '#']) => {
                    return Err(format!(
                        "topic `{raw}`: segment `{segment}` must be a literal or a whole `{{var}}`"
                    ));
                }
                None => Segment::Literal(segment.to_owned()),
            };

            segments.push(parsed);
        }

        Ok(Self { raw, segments })
    }
}

impl std::fmt::Display for TopicTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.raw)
    }
}

impl TopicTemplate {
    pub fn vars(&self) -> impl Iterator<Item = &str> {
        self.segments.iter().filter_map(|segment| match segment {
            Segment::Var(name) => Some(name.as_str()),
            Segment::Literal(_) => None,
        })
    }

    pub fn has_var(&self, name: &str) -> bool {
        self.vars().any(|var| var == name)
    }

    pub fn matches(&self, topic: &str) -> Option<TopicVars> {
        let parts: Vec<&str> = topic.split('/').collect();

        if parts.len() != self.segments.len() {
            return None;
        }

        let mut vars = TopicVars::new();

        for (segment, part) in self.segments.iter().zip(parts) {
            match segment {
                Segment::Literal(literal) if literal == part => {}
                Segment::Literal(_) => return None,
                Segment::Var(_) if part.is_empty() => return None,
                Segment::Var(name) => {
                    vars.insert(name.clone(), part.to_owned());
                }
            }
        }

        Some(vars)
    }

    pub fn render(&self, vars: &TopicVars) -> Result<String, String> {
        let parts =
            self.segments
                .iter()
                .map(|segment| match segment {
                    Segment::Literal(literal) => Ok(literal.as_str()),
                    Segment::Var(name) => vars.get(name).map(String::as_str).ok_or_else(|| {
                        format!("topic `{}` needs a value for `{{{name}}}`", self.raw)
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?;

        Ok(parts.join("/"))
    }

    pub fn filter(&self, vars: &TopicVars) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                Segment::Literal(literal) => literal.as_str(),
                Segment::Var(name) => vars.get(name).map_or("+", String::as_str),
            })
            .collect::<Vec<_>>()
            .join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template(raw: &str) -> TopicTemplate {
        TopicTemplate::try_from(raw.to_owned()).expect("template")
    }

    fn vars(pairs: &[(&str, &str)]) -> TopicVars {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn a_matched_topic_renders_back_to_itself() {
        let cases = [
            ("zigbee2mqtt/{name}", "zigbee2mqtt/front door"),
            (
                "{address}/{domain}/{object_id}/state",
                "apollo-plt-1/sensor/soil_moisture/state",
            ),
            ("valetudo/{address}/{leaf}", "valetudo/rockrobo/attributes"),
        ];

        for (raw, topic) in cases {
            let template = template(raw);
            let extracted = template.matches(topic).expect(topic);

            assert_eq!(template.render(&extracted).as_deref(), Ok(topic));
        }
    }

    #[test]
    fn a_topic_of_another_shape_does_not_match() {
        let template = template("zigbee2mqtt/{name}");

        assert_eq!(template.matches("zigbee2mqtt/bridge/devices"), None);
        assert_eq!(template.matches("esphome/node"), None);
        assert_eq!(template.matches("zigbee2mqtt/"), None);
    }

    #[test]
    fn unknown_vars_become_single_level_wildcards() {
        let template = template("valetudo/{address}/{leaf}");

        assert_eq!(template.filter(&TopicVars::new()), "valetudo/+/+");
        assert_eq!(
            template.filter(&vars(&[("address", "rockrobo")])),
            "valetudo/rockrobo/+"
        );
    }

    #[test]
    fn rendering_without_a_var_fails() {
        let error = template("valetudo/{address}/command")
            .render(&TopicVars::new())
            .expect_err("missing address");

        assert!(error.contains("{address}"), "{error}");
    }

    #[test]
    fn partial_placeholders_and_wildcards_are_rejected() {
        for raw in ["a/b{c}", "a/+/b", "a/#", "a/{}"] {
            assert!(TopicTemplate::try_from(raw.to_owned()).is_err(), "{raw}");
        }
    }
}
