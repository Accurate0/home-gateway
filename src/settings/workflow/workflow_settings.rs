use schemars::JsonSchema;
use serde::Deserialize;

use super::WorkflowTimerSettings;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WorkflowSettings {
    pub workers: usize,
    pub timers: WorkflowTimerSettings,
}
