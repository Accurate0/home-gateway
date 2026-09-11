use schemars::JsonSchema;
use serde::Deserialize;

/// On/off set command for a smart switch / plug.
#[derive(Debug, Deserialize, Clone, Copy, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SwitchState {
    On,
    Off,
    Toggle,
}
