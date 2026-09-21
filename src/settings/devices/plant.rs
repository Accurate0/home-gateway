use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PlantSensorSettings {
    #[allow(unused)]
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawPlantBlock {
    pub(crate) id: String,
}
