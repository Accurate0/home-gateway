use schemars::JsonSchema;
use serde::Deserialize;

use super::{Combinator, LeafCondition};
use crate::settings::DeviceAliases;

/// A boolean predicate evaluated against current device/sensor state. Either a
/// nested boolean combinator (`all`/`and`, `any`/`or`, `not`) or a leaf test.
#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum Condition {
    Combinator(Combinator),
    Leaf(LeafCondition),
}

impl Condition {
    pub fn references_namespace(&self, namespace: &str) -> bool {
        match self {
            Condition::Combinator(Combinator::All(conditions) | Combinator::Any(conditions)) => {
                conditions
                    .iter()
                    .any(|condition| condition.references_namespace(namespace))
            }
            Condition::Combinator(Combinator::Not(condition)) => {
                condition.references_namespace(namespace)
            }
            Condition::Leaf(LeafCondition::Var { var, .. }) => var.path.namespace() == namespace,
            Condition::Leaf(_) => false,
        }
    }

    pub(crate) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            Condition::Combinator(c) => c.resolve_devices(devices),
            Condition::Leaf(l) => l.resolve_devices(devices),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Condition::Combinator(c) => c.describe(),
            Condition::Leaf(l) => l.describe(),
        }
    }
}

pub(super) fn describe_join(conditions: &[Condition]) -> String {
    conditions
        .iter()
        .map(Condition::describe)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn resolve_opt(
    when: &mut Option<Condition>,
    devices: &DeviceAliases,
) -> Result<(), String> {
    if let Some(when) = when {
        when.resolve_devices(devices)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, File, FileFormat};

    #[derive(serde::Deserialize)]
    struct Wrap {
        when: Condition,
    }

    fn parse(yaml: &str) -> Condition {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize::<Wrap>()
            .unwrap()
            .when
    }

    #[test]
    fn nested_or_and_not_map_keys() {
        let cond = parse(
            r#"
when:
  or:
    - type: mode
      is: guest
    - and:
        - type: presence
          sensor: living-room
          present: true
        - not:
            type: mode
            is: vacation
"#,
        );
        assert_eq!(
            cond.describe(),
            "any[mode is guest, all[presence(living-room) is true, not(mode is vacation)]]"
        );
    }

    #[test]
    fn solar_home_assistant_and_smart_switch_leaves_parse() {
        let cond = parse(
            r#"
when:
  all:
    - type: solar
      metric: avg_15m
      op: gt
      value: 3000
    - type: home_assistant
      entity_id: binary_sensor.washer
      state: "off"
    - type: smart_switch
      device: "0x1"
      metric: power
      op: lt
      value: 5
"#,
        );
        assert_eq!(
            cond.describe(),
            "all[solar.avg_15m Gt 3000, ha(binary_sensor.washer) is off, switch(0x1).power Lt 5]"
        );
    }

    #[test]
    fn weather_leaf_parses_with_day() {
        let cond = parse(
            r#"
when:
  type: weather
  source: willyweather
  metric: max_temp
  day: tomorrow
  op: gte
  value: 35
"#,
        );
        assert_eq!(
            cond.describe(),
            "weather(willyweather).tomorrow_max_temp Gte 35"
        );
    }

    #[test]
    fn var_leaf_parses_with_filters() {
        let cond = parse(
            r#"
when:
  type: var
  var: "event.battery_percent | default(100)"
  op: lt
  value: 20
"#,
        );
        assert_eq!(
            cond.describe(),
            "event.battery_percent | default(100) Lt 20"
        );
        assert!(cond.references_namespace("event"));
        assert!(!cond.references_namespace("input"));
    }

    #[test]
    fn all_any_aliases_match_and_or() {
        let with_all = parse("when:\n  all:\n    - type: mode\n      is: guest\n");
        let with_and = parse("when:\n  and:\n    - type: mode\n      is: guest\n");
        assert_eq!(with_all.describe(), with_and.describe());
    }
}
