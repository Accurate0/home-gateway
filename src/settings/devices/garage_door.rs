use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct GarageDoorSettings {
    pub name: String,
    pub id: String,
    pub address: String,
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub struct RawGarageDoorSettings {
    name: String,
}

impl RawGarageDoorSettings {
    pub(crate) fn resolve(self, id: &str, address: &str) -> GarageDoorSettings {
        GarageDoorSettings {
            name: self.name,
            id: id.to_owned(),
            address: address.to_owned(),
        }
    }
}
