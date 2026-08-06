use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwitchRole {
    Light,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RawSmartSwitchBlock {
    pub(crate) name: String,
    #[serde(default, rename = "as")]
    pub(crate) role: Option<SwitchRole>,
}
