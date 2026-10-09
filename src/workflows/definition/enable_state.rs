use schemars::JsonSchema;
use serde::Deserialize;

/// Target enablement for a `set_workflows_enabled` step. `Toggle` flips the
/// whole tagged set together, based on whether any member is currently enabled.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnableState {
    Enabled,
    Disabled,
    Toggle,
}
