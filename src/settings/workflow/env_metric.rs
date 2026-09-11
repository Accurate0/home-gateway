use schemars::JsonSchema;
use serde::Deserialize;

/// Which reading of an environment sensor a condition compares against.
#[derive(Debug, Deserialize, Clone, Copy, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnvMetric {
    Temperature,
    Humidity,
    Pressure,
    Lux,
    UvIndex,
}
