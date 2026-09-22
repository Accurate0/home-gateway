use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::mode::Mode;
use crate::settings::enabled_state::EnabledState;
use crate::timedelta_format::time_delta_from_str;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct VacationSettings {
    pub state: EnabledState,
    pub modes: Vec<Mode>,
    #[serde(deserialize_with = "time_delta_from_str::deserialize")]
    #[schemars(with = "String")]
    pub window: TimeDelta,
    #[serde(deserialize_with = "time_delta_from_str::deserialize")]
    #[schemars(with = "String")]
    pub jitter: TimeDelta,
    pub min_observations: i64,
    pub seed: u64,
}

impl VacationSettings {
    pub fn arms_on(&self, mode: &Mode) -> bool {
        self.state.is_enabled() && self.modes.contains(mode)
    }
}
