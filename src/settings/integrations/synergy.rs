use crate::lua::CallTarget;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SynergySettings {
    pub parser: CallTarget,
}
