use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use chrono_tz::Australia::Perth;
use serde::Deserialize;
use sunrise::{Coordinates, SolarDay, SolarEvent};

use crate::settings::location::LocationSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SunPeriod {
    Day,
    Night,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SunTransition {
    Sunrise,
    Sunset,
}

impl SunTransition {
    pub fn as_str(&self) -> &'static str {
        match self {
            SunTransition::Sunrise => "sunrise",
            SunTransition::Sunset => "sunset",
        }
    }
}

fn sun_times(loc: LocationSettings, date: chrono::NaiveDate) -> (DateTime<Utc>, DateTime<Utc>) {
    let coord =
        Coordinates::new(loc.latitude, loc.longitude).expect("invalid location coordinates");
    let day = SolarDay::new(coord, date);
    (
        day.event_time(SolarEvent::Sunrise)
            .expect("no sunrise at configured location"),
        day.event_time(SolarEvent::Sunset)
            .expect("no sunset at configured location"),
    )
}

fn event_time(
    loc: LocationSettings,
    date: chrono::NaiveDate,
    transition: SunTransition,
    offset: TimeDelta,
) -> DateTime<Utc> {
    let (rise, set) = sun_times(loc, date);
    let base = match transition {
        SunTransition::Sunrise => rise,
        SunTransition::Sunset => set,
    };
    base + offset
}

pub fn current_period(loc: LocationSettings, now: DateTime<Utc>, offset: TimeDelta) -> SunPeriod {
    let date = now.with_timezone(&Perth).date_naive();
    let rise = event_time(loc, date, SunTransition::Sunrise, offset);
    let set = event_time(loc, date, SunTransition::Sunset, offset);
    if now >= rise && now < set {
        SunPeriod::Day
    } else {
        SunPeriod::Night
    }
}

pub fn next_transition_at(
    loc: LocationSettings,
    now: DateTime<Utc>,
    transition: SunTransition,
    offset: TimeDelta,
) -> DateTime<Utc> {
    let today = now.with_timezone(&Perth).date_naive();
    let tomorrow = today.succ_opt().unwrap_or(today);

    let today_at = event_time(loc, today, transition, offset);
    if today_at > now {
        today_at
    } else {
        event_time(loc, tomorrow, transition, offset)
    }
}

pub fn previous_transition_at(
    loc: LocationSettings,
    now: DateTime<Utc>,
    transition: SunTransition,
    offset: TimeDelta,
) -> DateTime<Utc> {
    let today = now.with_timezone(&Perth).date_naive();
    let yesterday = today.pred_opt().unwrap_or(today);

    let today_at = event_time(loc, today, transition, offset);
    if today_at <= now {
        today_at
    } else {
        event_time(loc, yesterday, transition, offset)
    }
}

pub fn next_transition(
    loc: LocationSettings,
    now: DateTime<Utc>,
    transition: SunTransition,
    offset: TimeDelta,
) -> Duration {
    (next_transition_at(loc, now, transition, offset) - now)
        .to_std()
        .unwrap_or(Duration::ZERO)
}

pub fn should_catch_up(
    previous: DateTime<Utc>,
    last_fired: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
    grace: TimeDelta,
) -> bool {
    if last_fired.is_some_and(|fired| fired >= previous) {
        return false;
    }

    now - previous <= grace
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOC: LocationSettings = LocationSettings {
        latitude: -32.135429,
        longitude: 115.865509,
    };

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn midday_is_day_midnight_is_night() {
        let midday = utc("2026-07-11T04:00:00Z");
        let midnight = utc("2026-07-11T16:00:00Z");
        assert_eq!(
            current_period(LOC, midday, TimeDelta::zero()),
            SunPeriod::Day
        );
        assert_eq!(
            current_period(LOC, midnight, TimeDelta::zero()),
            SunPeriod::Night
        );
    }

    #[test]
    fn next_transition_is_positive() {
        let now = utc("2026-07-11T04:00:00Z");
        let delay = next_transition(LOC, now, SunTransition::Sunset, TimeDelta::zero());
        assert!(delay > Duration::ZERO);
    }

    #[test]
    fn offset_shifts_period_boundary() {
        let set = event_time(
            LOC,
            now_date("2026-07-11"),
            SunTransition::Sunset,
            TimeDelta::zero(),
        );
        let just_before = set - TimeDelta::minutes(10);
        assert_eq!(
            current_period(LOC, just_before, TimeDelta::zero()),
            SunPeriod::Day
        );
        assert_eq!(
            current_period(LOC, just_before, TimeDelta::minutes(-30)),
            SunPeriod::Night
        );
    }

    #[test]
    fn offset_shifts_next_transition() {
        let now = utc("2026-07-11T04:00:00Z");
        let base = next_transition(LOC, now, SunTransition::Sunset, TimeDelta::zero());
        let earlier = next_transition(LOC, now, SunTransition::Sunset, TimeDelta::minutes(-30));
        assert!(earlier < base);
    }

    #[test]
    fn previous_transition_is_in_the_last_day() {
        let now = utc("2026-07-11T04:00:00Z");
        let prev = previous_transition_at(LOC, now, SunTransition::Sunset, TimeDelta::zero());
        assert!(prev <= now);
        assert!(now - prev < TimeDelta::hours(25));
    }

    #[test]
    fn previous_and_next_bracket_now() {
        let now = utc("2026-07-11T12:00:00Z");
        let prev =
            previous_transition_at(LOC, now, SunTransition::Sunrise, TimeDelta::minutes(-30));
        let next = next_transition_at(LOC, now, SunTransition::Sunrise, TimeDelta::minutes(-30));
        assert!(prev <= now);
        assert!(next > now);
        assert_eq!(
            next_transition_at(
                LOC,
                prev - TimeDelta::seconds(1),
                SunTransition::Sunrise,
                TimeDelta::minutes(-30)
            ),
            prev
        );
    }

    #[test]
    fn catch_up_only_for_recent_unfired_transitions() {
        let now = utc("2026-07-11T12:00:00Z");
        let grace = TimeDelta::hours(2);
        let recent = now - TimeDelta::minutes(30);
        let stale = now - TimeDelta::hours(6);

        assert!(should_catch_up(recent, None, now, grace));
        assert!(!should_catch_up(stale, None, now, grace));
        assert!(!should_catch_up(recent, Some(recent), now, grace));
        assert!(should_catch_up(
            recent,
            Some(recent - TimeDelta::days(1)),
            now,
            grace
        ));
    }

    fn now_date(s: &str) -> chrono::NaiveDate {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }
}
