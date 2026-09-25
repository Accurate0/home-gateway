use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EnergyInterval {
    pub used: f64,
    pub exported: f64,
    pub at: i64,
}

impl EnergyInterval {
    pub fn time(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(self.at, 0)
    }
}
