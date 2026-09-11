use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawLightBlock {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) entity: Option<String>,
}
