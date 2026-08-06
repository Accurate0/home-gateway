//! Standard cron schedules for [`Cron`](crate::settings::TriggerMatcher::Cron)
//! triggers.
//!
//! Wraps [`croner::Cron`] (5-field `min hour dom month dow`) and evaluates it in
//! the home's local timezone, so `0 20 * * THU` means 8pm Perth time regardless
//! of the host clock's zone.

use std::time::Duration;

use chrono::Utc;
use chrono_tz::Australia::Perth;
use croner::{Cron, errors::CronError};
use serde::Deserialize;

/// Deserialized straight from a cron string by croner's `serde` feature, so the
/// config accepts the full readable syntax: day/month aliases (`THU`, `JAN`),
/// nicknames (`@weekly`, `@daily`), ranges (`MON-FRI`) and steps (`*/15`).
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
#[schemars(with = "String")]
pub struct CronSchedule(Cron);

impl CronSchedule {
    pub fn expression(&self) -> String {
        self.0.pattern.to_string()
    }

    /// Time from now until the next occurrence strictly after now.
    pub fn time_until_next(&self) -> Result<Duration, CronError> {
        let now = Utc::now().with_timezone(&Perth);
        let next = self.0.find_next_occurrence(&now, false)?;
        // `next` is strictly after `now`, so the delta is always non-negative
        Ok((next - now).to_std().unwrap_or(Duration::ZERO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_cron() {
        let yaml = "\"0 20 * * THU\"";
        let schedule: CronSchedule = serde_yaml::from_str(yaml).unwrap();
        // a valid schedule always resolves a next occurrence
        assert!(schedule.time_until_next().is_ok());
    }

    #[test]
    fn rejects_invalid_cron() {
        let yaml = "\"not a cron\"";
        assert!(serde_yaml::from_str::<CronSchedule>(yaml).is_err());
    }
}
