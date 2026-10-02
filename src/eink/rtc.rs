use chrono::{DateTime, TimeDelta, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcSync {
    pub drift: Option<TimeDelta>,
}

pub fn rtc_sync(
    reported: Option<DateTime<Utc>>,
    synced_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
    interval: TimeDelta,
) -> Option<RtcSync> {
    let reported = reported?;

    let Some(synced_at) = synced_at else {
        return Some(RtcSync { drift: None });
    };

    if reported < synced_at {
        return Some(RtcSync { drift: None });
    }

    if now - synced_at < interval {
        return None;
    }

    Some(RtcSync {
        drift: Some(reported - now),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    fn at(day: u32, hour: u32, second: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, day, hour, 0, second).unwrap()
    }

    fn daily() -> TimeDelta {
        TimeDelta::hours(24)
    }

    #[test]
    fn firmware_that_reports_no_clock_is_never_synced() {
        assert_eq!(rtc_sync(None, None, at(2, 8, 0), daily()), None);
        assert_eq!(rtc_sync(None, Some(at(1, 8, 0)), at(3, 8, 0), daily()), None);
    }

    #[test]
    fn a_display_that_was_never_synced_is_synced_without_a_drift() {
        let epoch = Utc.timestamp_opt(12, 0).unwrap();

        assert_eq!(
            rtc_sync(Some(epoch), None, at(2, 8, 0), daily()),
            Some(RtcSync { drift: None })
        );
    }

    #[test]
    fn a_display_synced_within_the_interval_is_left_alone() {
        assert_eq!(
            rtc_sync(Some(at(2, 7, 3)), Some(at(1, 8, 0)), at(2, 7, 0), daily()),
            None
        );
    }

    #[test]
    fn a_due_sync_measures_a_fast_clock() {
        assert_eq!(
            rtc_sync(Some(at(2, 8, 42)), Some(at(1, 8, 0)), at(2, 8, 0), daily()),
            Some(RtcSync {
                drift: Some(TimeDelta::seconds(42))
            })
        );
    }

    #[test]
    fn a_due_sync_measures_a_slow_clock() {
        assert_eq!(
            rtc_sync(Some(at(2, 8, 0)), Some(at(1, 8, 0)), at(2, 8, 30), daily()),
            Some(RtcSync {
                drift: Some(TimeDelta::seconds(-30))
            })
        );
    }

    #[test]
    fn a_clock_behind_the_last_sync_was_lost_and_is_set_again() {
        let epoch = Utc.timestamp_opt(12, 0).unwrap();

        assert_eq!(
            rtc_sync(Some(epoch), Some(at(2, 6, 0)), at(2, 8, 0), daily()),
            Some(RtcSync { drift: None })
        );
    }
}
