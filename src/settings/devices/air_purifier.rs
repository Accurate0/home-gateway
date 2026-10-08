use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct AirPurifierSettings {
    pub name: String,
    pub id: String,
    pub address: String,
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawAirPurifierBlock {
    name: String,
}

impl RawAirPurifierBlock {
    pub(crate) fn resolve(self, id: &str, address: &str) -> AirPurifierSettings {
        AirPurifierSettings {
            name: self.name,
            id: id.to_owned(),
            address: address.to_owned(),
        }
    }
}
