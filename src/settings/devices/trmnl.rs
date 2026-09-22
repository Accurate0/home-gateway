use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawTrmnlBlock {
    pub(crate) name: String,
}

#[derive(Debug, Clone)]
pub struct TrmnlDeviceSettings {
    pub id: String,
    pub name: String,
}
