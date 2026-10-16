use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct DoorSettings {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub struct RawDoorSettings {
    name: String,
}

impl RawDoorSettings {
    pub(crate) fn resolve(self, id: &str) -> DoorSettings {
        DoorSettings {
            name: self.name,
            id: id.to_owned(),
        }
    }
}
