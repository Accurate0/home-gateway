use std::time::Duration;

use chrono::{DateTime, Utc};

use super::StepOutcome;

#[derive(Debug, Clone, PartialEq)]
pub struct StepTrace {
    pub depth: u8,
    pub kind: String,
    pub outcome: StepOutcome,
    pub guard: Option<String>,
    pub detail: Option<String>,
    pub error: Option<String>,
    pub duration: Duration,
    pub at: DateTime<Utc>,
}
