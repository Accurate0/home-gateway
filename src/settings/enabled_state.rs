use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnabledState {
    Enabled,
    Disabled,
}

impl EnabledState {
    pub fn is_enabled(self) -> bool {
        self == Self::Enabled
    }
}

impl std::fmt::Display for EnabledState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Enabled => write!(f, "enabled"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}
