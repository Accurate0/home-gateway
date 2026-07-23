use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{settings::yes, timedelta_format::time_delta_from_str};

pub(crate) fn no_throttle() -> TimeDelta {
    TimeDelta::zero()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct EntitySettings {
    pub id: String,
    #[serde(default = "yes")]
    pub log: bool,
    #[serde(default = "yes")]
    pub latest_state: bool,
    #[serde(default = "no_throttle", with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub throttle: TimeDelta,
}

impl Default for EntitySettings {
    fn default() -> Self {
        Self {
            id: String::new(),
            log: true,
            latest_state: true,
            throttle: no_throttle(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct HomeAssistantSettings {
    #[serde(default)]
    pub entities: Vec<EntitySettings>,
}

impl HomeAssistantSettings {
    pub fn for_entity(&self, entity_id: &str) -> EntitySettings {
        self.entities
            .iter()
            .find(|entity| matches_pattern(&entity.id, entity_id))
            .cloned()
            .unwrap_or_default()
    }
}

fn matches_pattern(pattern: &str, entity_id: &str) -> bool {
    match pattern.split_once('*') {
        Some((prefix, suffix)) => {
            entity_id.len() >= prefix.len() + suffix.len()
                && entity_id.starts_with(prefix)
                && entity_id.ends_with(suffix)
        }
        None => pattern == entity_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> HomeAssistantSettings {
        HomeAssistantSettings {
            entities: vec![
                EntitySettings {
                    id: "sensor.apollo_mtr_1_living_room_target_*".to_owned(),
                    log: false,
                    throttle: TimeDelta::seconds(30),
                    ..Default::default()
                },
                EntitySettings {
                    id: "sensor.noisy".to_owned(),
                    throttle: TimeDelta::seconds(5),
                    ..Default::default()
                },
            ],
        }
    }

    #[test]
    fn matches_wildcard_entities() {
        let entity = settings().for_entity("sensor.apollo_mtr_1_living_room_target_1_angle");
        assert!(!entity.log);
        assert!(entity.latest_state);
        assert_eq!(entity.throttle, TimeDelta::seconds(30));
    }

    #[test]
    fn matches_exact_entities() {
        let entity = settings().for_entity("sensor.noisy");
        assert!(entity.log);
        assert_eq!(entity.throttle, TimeDelta::seconds(5));
    }

    #[test]
    fn unlisted_entities_use_defaults() {
        let entity = settings().for_entity("sensor.noisy_neighbour");
        assert!(entity.log);
        assert!(entity.latest_state);
        assert_eq!(entity.throttle, TimeDelta::zero());
    }
}
