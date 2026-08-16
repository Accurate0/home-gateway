//! Standard cron schedules for [`Cron`](crate::settings::TriggerMatcher::Cron)
//! triggers.
//!
//! Wraps [`croner::Cron`] (5-field `min hour dom month dow`) and evaluates it in
//! the home's local timezone, so `0 20 * * THU` means 8pm Perth time regardless
//! of the host clock's zone.

use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use chrono_tz::Australia::Perth;
use chrono_tz::Tz;
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
    pub fn parse(expression: &str) -> Result<Self, CronError> {
        expression.parse().map(Self)
    }

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

    pub fn secs_until_next_from(
        &self,
        from: DateTime<Tz>,
        grace: TimeDelta,
    ) -> Result<u32, CronError> {
        let after = from + grace;
        let next = self.0.find_next_occurrence(&after, false)?;

        Ok((next - from).num_seconds().max(0) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

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

    fn hourly() -> CronSchedule {
        serde_yaml::from_str("\"0 * * * *\"").unwrap()
    }

    fn perth(hour: u32, minute: u32, second: u32) -> DateTime<Tz> {
        Perth
            .with_ymd_and_hms(2026, 8, 16, hour, minute, second)
            .unwrap()
    }

    #[test]
    fn a_wake_inside_the_grace_targets_the_following_slot() {
        assert_eq!(
            hourly()
                .secs_until_next_from(perth(9, 58, 0), TimeDelta::minutes(10))
                .unwrap(),
            62 * 60
        );
    }

    #[test]
    fn a_wake_on_the_slot_targets_the_following_slot() {
        assert_eq!(
            hourly()
                .secs_until_next_from(perth(10, 0, 0), TimeDelta::minutes(10))
                .unwrap(),
            60 * 60
        );
    }

    #[test]
    fn a_wake_past_the_grace_targets_the_next_slot() {
        assert_eq!(
            hourly()
                .secs_until_next_from(perth(10, 3, 0), TimeDelta::minutes(10))
                .unwrap(),
            57 * 60
        );
    }

    #[test]
    fn a_zero_grace_takes_the_very_next_slot() {
        assert_eq!(
            hourly()
                .secs_until_next_from(perth(9, 58, 0), TimeDelta::zero())
                .unwrap(),
            2 * 60
        );
    }
}
