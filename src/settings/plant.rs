use serde::Deserialize;

use super::DeviceAliases;
use super::workflow::WorkflowSettings;

/// Default esphome sensor entity for a plant: the Apollo PLT-1 publishes soil
/// moisture as a percentage on `<node>/sensor/soil_moisture/state`.
fn default_entity() -> String {
    "soil_moisture".to_string()
}

/// A threshold test against a sensor reading. Workflows fire on the rising edge
/// of the test becoming true, so a reading staying below a threshold only
/// triggers once rather than on every interval.
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PlantCondition {
    /// reading > value
    Above(f64),
    /// reading < value
    Below(f64),
    /// reading == value
    Equal(f64),
}

impl PlantCondition {
    pub fn is_satisfied(&self, reading: f64) -> bool {
        match self {
            PlantCondition::Above(v) => reading > *v,
            PlantCondition::Below(v) => reading < *v,
            PlantCondition::Equal(v) => reading == *v,
        }
    }
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
pub struct PlantActionSettings {
    /// esphome sensor object_id this threshold watches (e.g. `soil_moisture`).
    pub entity: String,
    pub when: PlantCondition,
    pub workflow: WorkflowSettings,
}

#[derive(Debug, Clone)]
pub struct PlantSensorSettings {
    pub id: String,
    pub actions: Vec<PlantActionSettings>,
}

#[derive(Debug, Deserialize, Clone)]
struct RawPlantAction {
    #[serde(default = "default_entity")]
    entity: String,
    when: PlantCondition,
    #[serde(flatten)]
    workflow: WorkflowSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct RawPlantSensor {
    id: String,
    source: PlantSource,
    #[serde(default)]
    actions: Vec<RawPlantAction>,
}

impl RawPlantSensor {
    /// Resolve into `(node, settings)` for the runtime sensor map.
    pub(super) fn resolve(
        self,
        devices: &DeviceAliases,
    ) -> Result<(String, PlantSensorSettings), String> {
        let identifier = self.source.identifier();
        let actions = self
            .actions
            .into_iter()
            .map(|mut a| {
                for step in &mut a.workflow.run {
                    step.resolve_devices(devices)?;
                }
                Ok(PlantActionSettings {
                    entity: a.entity,
                    when: a.when,
                    workflow: a.workflow,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok((
            identifier,
            PlantSensorSettings {
                id: self.id,
                actions,
            },
        ))
    }
}
