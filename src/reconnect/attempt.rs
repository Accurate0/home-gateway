use std::fmt::Display;
use std::time::Duration;

pub struct Attempt {
    pub delay: Duration,
    pub failures: u32,
    pub log: bool,
    pub last_log: bool,
}

impl Attempt {
    pub fn log_failure(&self, subject: impl Display, error: impl Display) {
        if !self.log {
            tracing::debug!(
                "{subject} error after {} attempts, reconnecting in {:?}: {error}",
                self.failures,
                self.delay
            );
            return;
        }

        tracing::error!("{subject} error, reconnecting in {:?}: {error}", self.delay);

        if self.last_log {
            tracing::error!(
                "{subject} failed {} times, further errors logged at debug until it reconnects",
                self.failures
            );
        }
    }
}
