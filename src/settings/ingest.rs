use schemars::JsonSchema;
use serde::Deserialize;

use crate::lua::Script;

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct IngestSettings {
    #[serde(default)]
    pub sources: Vec<IngestSource>,
}

impl IngestSettings {
    pub fn source(&self, name: &str) -> Option<&IngestSource> {
        self.sources.iter().find(|source| source.name == name)
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct IngestSource {
    pub name: String,
    pub script: Script,
}
