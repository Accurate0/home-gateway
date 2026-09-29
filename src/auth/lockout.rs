use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use moka::future::Cache;

use crate::settings::AuthLockoutSettings;

#[derive(Clone)]
pub struct AuthLockout {
    attempts: u32,
    failures: Cache<IpAddr, Arc<AtomicU32>>,
}

impl AuthLockout {
    pub fn new(settings: &AuthLockoutSettings) -> Self {
        let failures = Cache::builder()
            .max_capacity(settings.capacity)
            .time_to_live(settings.window())
            .build();

        Self {
            attempts: settings.attempts,
            failures,
        }
    }

    pub async fn is_locked(&self, ip: IpAddr) -> bool {
        self.failures
            .get(&ip)
            .await
            .is_some_and(|failures| failures.load(Ordering::Relaxed) >= self.attempts)
    }

    pub async fn record_failure(&self, ip: IpAddr) {
        let failures = self
            .failures
            .get_with(ip, async { Arc::new(AtomicU32::new(0)) })
            .await;

        let count = failures.fetch_add(1, Ordering::Relaxed) + 1;

        if count == self.attempts {
            tracing::warn!("locking out {ip} after {count} failed auth attempts");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    fn lockout(attempts: u32) -> AuthLockout {
        AuthLockout::new(&AuthLockoutSettings {
            attempts,
            window: TimeDelta::minutes(5),
            capacity: 16,
        })
    }

    #[tokio::test]
    async fn an_address_is_locked_once_it_reaches_the_attempt_limit() {
        let lockout = lockout(3);
        let ip: IpAddr = "203.0.113.7".parse().unwrap();

        lockout.record_failure(ip).await;
        lockout.record_failure(ip).await;

        assert!(!lockout.is_locked(ip).await);

        lockout.record_failure(ip).await;

        assert!(lockout.is_locked(ip).await);
    }

    #[tokio::test]
    async fn failures_from_one_address_do_not_lock_another() {
        let lockout = lockout(1);
        let noisy: IpAddr = "203.0.113.7".parse().unwrap();
        let quiet: IpAddr = "203.0.113.8".parse().unwrap();

        lockout.record_failure(noisy).await;

        assert!(lockout.is_locked(noisy).await);
        assert!(!lockout.is_locked(quiet).await);
    }
}
