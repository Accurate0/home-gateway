use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackState {
    Started,
    Stopped,
    Paused,
    Resumed,
}

impl PlaybackState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlaybackState::Started => "started",
            PlaybackState::Stopped => "stopped",
            PlaybackState::Paused => "paused",
            PlaybackState::Resumed => "resumed",
        }
    }
}
