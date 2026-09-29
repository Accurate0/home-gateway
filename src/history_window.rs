use chrono::{DateTime, TimeDelta, Utc};

pub fn clamp_since(since: DateTime<Utc>, now: DateTime<Utc>, window: TimeDelta) -> DateTime<Utc> {
    since.max(now - window)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-07-28T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn since_older_than_the_window_is_clamped() {
        let window = TimeDelta::days(90);

        assert_eq!(
            clamp_since(now() - TimeDelta::days(400), now(), window),
            now() - window
        );
    }

    #[test]
    fn since_inside_the_window_is_kept() {
        let recent = now() - TimeDelta::days(1);

        assert_eq!(clamp_since(recent, now(), TimeDelta::days(90)), recent);
    }
}
