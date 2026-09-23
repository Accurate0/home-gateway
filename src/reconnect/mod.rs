mod attempt;

pub use attempt::Attempt;

use std::time::Duration;

use crate::settings::ReconnectSettings;

pub struct Reconnect {
    settings: ReconnectSettings,
    delay: Duration,
    failures: u32,
}

impl Reconnect {
    pub fn new(settings: ReconnectSettings) -> Self {
        Self {
            settings,
            delay: settings.backoff.min(),
            failures: 0,
        }
    }

    pub fn connected(&mut self) -> bool {
        let recovered = self.failures > 0;

        self.delay = self.settings.backoff.min();
        self.failures = 0;

        recovered
    }

    pub fn failed(&mut self) -> Attempt {
        let delay = self.delay;

        self.failures += 1;
        self.delay = (delay * 2).min(self.settings.backoff.max());

        Attempt {
            delay,
            failures: self.failures,
            log: self.failures <= self.settings.log_attempts,
            last_log: self.failures == self.settings.log_attempts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeDelta;

    use crate::settings::BackoffSettings;

    fn settings() -> ReconnectSettings {
        ReconnectSettings {
            backoff: BackoffSettings {
                min: TimeDelta::seconds(5),
                max: TimeDelta::seconds(20),
            },
            log_attempts: 2,
        }
    }

    #[test]
    fn the_delay_doubles_up_to_the_maximum() {
        let mut reconnect = Reconnect::new(settings());

        assert_eq!(reconnect.failed().delay, Duration::from_secs(5));
        assert_eq!(reconnect.failed().delay, Duration::from_secs(10));
        assert_eq!(reconnect.failed().delay, Duration::from_secs(20));
        assert_eq!(reconnect.failed().delay, Duration::from_secs(20));
    }

    #[test]
    fn logging_stops_after_the_configured_attempts() {
        let mut reconnect = Reconnect::new(settings());

        assert!(reconnect.failed().log);

        let last = reconnect.failed();

        assert!(last.log);
        assert!(last.last_log);
        assert!(!reconnect.failed().log);
    }

    #[test]
    fn a_connection_resets_the_delay_and_reports_recovery() {
        let mut reconnect = Reconnect::new(settings());

        assert!(!reconnect.connected());

        reconnect.failed();
        reconnect.failed();

        assert!(reconnect.connected());
        assert_eq!(reconnect.failed().delay, Duration::from_secs(5));
    }
}
