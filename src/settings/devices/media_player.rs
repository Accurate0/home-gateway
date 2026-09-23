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
    pub address: String,
}

impl RawMediaPlayerBlock {
    pub fn resolve(self, id: &str, address: &str) -> MediaPlayerSettings {
        MediaPlayerSettings {
            id: id.to_owned(),
            name: self.name,
            address: address.to_owned(),
        }
    }
}
