use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    Temperature,
    Humidity,
    Pressure,
    Lux,
    UvIndex,
    Pm25,
    VocIndex,
}

#[derive(Debug, Clone)]
pub struct EnvironmentSensorSettings {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawEnvironmentBlock {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) name: Option<String>,
}
