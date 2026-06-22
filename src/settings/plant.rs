use serde::Deserialize;

use super::DeviceAliases;

/// Default esphome sensor entity for a plant: the Apollo PLT-1 publishes soil
/// moisture as a percentage on `<node>/sensor/soil_moisture/state`.
fn default_entities() -> Vec<String> {
    vec!["soil_moisture".to_string()]
}

/// Where a plant sensor's readings come from. Only esphome is supported; the
/// variant is explicit so the map key's meaning (node name) is never implied.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PlantSource {
    Esphome { node: String },
}

impl PlantSource {
    fn identifier(&self) -> String {
        match self {
            PlantSource::Esphome { node } => node.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlantSensorSettings {
    #[allow(unused)]
    pub id: String,
    /// esphome sensor object_ids to subscribe to and publish as `Environment`
    /// events (thresholds live in `triggers:`).
    pub entities: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct RawPlantSensor {
    id: String,
    source: PlantSource,
    #[serde(default = "default_entities")]
    entities: Vec<String>,
}

impl RawPlantSensor {
    /// Resolve into `(node, settings)` for the runtime sensor map.
    pub(super) fn resolve(
        self,
        _devices: &DeviceAliases,
    ) -> Result<(String, PlantSensorSettings), String> {
        let identifier = self.source.identifier();

        Ok((
            identifier,
            PlantSensorSettings {
                id: self.id,
                entities: self.entities,
            },
        ))
    }
}
