use crate::settings::enabled_state::EnabledState;
use crate::settings::integrations::unraid_websocket::UnraidWebsocketSettings;
use crate::timedelta_format::time_delta_from_str;
use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UnraidSettings {
    pub state: EnabledState,
    pub url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(with = "time_delta_from_str")]
    #[schemars(with = "String")]
    pub refresh: TimeDelta,
    pub websocket: UnraidWebsocketSettings,
    pub notification_backfill: u16,
}

impl UnraidSettings {
    pub fn refresh(&self) -> Duration {
        self.refresh.to_std().unwrap_or_default()
    }
}
