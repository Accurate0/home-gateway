use super::{ArrayState, ParityStatus};

#[derive(Debug, Clone, PartialEq)]
pub struct ArraySnapshot {
    pub state: ArrayState,
    pub kilobytes_free: i64,
    pub kilobytes_used: i64,
    pub kilobytes_total: i64,
    pub parity_status: ParityStatus,
    pub parity_progress: Option<i64>,
    pub parity_errors: Option<i64>,
    pub parity_running: bool,
}

impl ArraySnapshot {
    pub fn usage_percent(&self) -> Option<f64> {
        if self.kilobytes_total <= 0 {
            return None;
        }

        Some(self.kilobytes_used as f64 / self.kilobytes_total as f64 * 100.0)
    }
}
