use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct LocationSettings {
    pub latitude: f64,
    pub longitude: f64,
}
