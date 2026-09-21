use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct HomeAssistantEntities {
    templates: Vec<String>,
}

impl HomeAssistantEntities {
    pub fn resolve(&self, address: &str) -> Vec<String> {
        let name = address
            .split_once('.')
            .map_or(address, |(_, object_id)| object_id);

        self.templates
            .iter()
            .map(|template| template.replace("{name}", name))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_are_filled_with_the_address_object_id() {
        let entities = HomeAssistantEntities {
            templates: vec![
                "sensor.{name}_status".to_owned(),
                "binary_sensor.kitchen_{name}_dock".to_owned(),
            ],
        };

        assert_eq!(
            entities.resolve("vacuum.robot"),
            ["sensor.robot_status", "binary_sensor.kitchen_robot_dock"]
        );
    }
}
