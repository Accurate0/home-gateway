use std::fmt;

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlagState {
    Ready,
    Changed,
    Stale,
    Error,
}

impl FeatureFlagState {
    pub fn should_reevaluate(&self) -> bool {
        matches!(self, Self::Ready | Self::Changed)
    }
}

impl fmt::Display for FeatureFlagState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Ready => "ready",
            Self::Changed => "changed",
            Self::Stale => "stale",
            Self::Error => "error",
        };

        f.write_str(name)
    }
}
