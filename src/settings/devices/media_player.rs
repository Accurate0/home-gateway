use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawMediaPlayerBlock {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct MediaPlayerSettings {
    pub id: String,
    pub name: String,
    pub entity_id: String,
}

impl RawMediaPlayerBlock {
    pub fn resolve(self, id: &str, entity_id: &str) -> MediaPlayerSettings {
        MediaPlayerSettings {
            id: id.to_owned(),
            name: self.name,
            entity_id: entity_id.to_owned(),
        }
    }
}
