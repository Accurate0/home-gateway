use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub enum RenderSchedule {
    Now,
    After(Duration),
}

pub fn render_schedule(
    next_wake_at: Option<chrono::DateTime<chrono::Utc>>,
    lead: chrono::TimeDelta,
    now: chrono::DateTime<chrono::Utc>,
) -> RenderSchedule {
    let Some(next_wake_at) = next_wake_at else {
        return RenderSchedule::Now;
    };

    match (next_wake_at - lead - now).to_std() {
        Ok(delay) => RenderSchedule::After(delay),
        Err(_) => RenderSchedule::Now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wake_beyond_the_lead_is_scheduled() {
        let now = chrono::Utc::now();
        let lead = chrono::TimeDelta::minutes(15);

        assert_eq!(
            render_schedule(Some(now + chrono::TimeDelta::hours(1)), lead, now),
            RenderSchedule::After(Duration::from_secs(45 * 60))
        );
    }

    #[test]
    fn a_wake_inside_the_lead_renders_now() {
        let now = chrono::Utc::now();
        let lead = chrono::TimeDelta::minutes(15);

        assert_eq!(
            render_schedule(Some(now + chrono::TimeDelta::minutes(5)), lead, now),
            RenderSchedule::Now
        );
    }

    #[test]
    fn a_wake_in_the_past_renders_now() {
        let now = chrono::Utc::now();
        let lead = chrono::TimeDelta::minutes(15);

        assert_eq!(
            render_schedule(Some(now - chrono::TimeDelta::days(3)), lead, now),
            RenderSchedule::Now
        );
    }

    #[test]
    fn a_display_that_never_polled_renders_now() {
        let now = chrono::Utc::now();

        assert_eq!(
            render_schedule(None, chrono::TimeDelta::minutes(15), now),
            RenderSchedule::Now
        );
    }
}
