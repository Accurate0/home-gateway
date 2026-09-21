use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PresenceSettings {
    #[allow(unused)]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawPresenceBlock {
    pub(crate) name: String,
}
