use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PlantSensorSettings {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawPlantBlock {
    pub(crate) id: String,
    pub(crate) name: String,
}
