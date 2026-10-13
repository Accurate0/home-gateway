mod lock_settings;
pub mod lua;

pub use lock_settings::{LockSettings, RawLockSettings};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::device_command::CommandOrigin;
use crate::event_bus::{EventBus, EventBusMessage};
use crate::intent::{self, Attribution};
use crate::repo::device_lock::{DeviceLock, DeviceLockSource};
use crate::repo::intent::DeviceReport;
use crate::repo::{DeviceLockRepo, IntentRepo, RepoRegistry};

pub struct LockedDevice<'a> {
    pub device_id: &'a str,
    pub address: &'a str,
    pub settings: &'a LockSettings,
}

pub struct DeviceLocks<'a> {
    locks: &'a DeviceLockRepo,
    intents: &'a IntentRepo,
    event_bus: &'a EventBus,
}

impl<'a> DeviceLocks<'a> {
    pub fn new(repos: &'a RepoRegistry, event_bus: &'a EventBus) -> Self {
        Self {
            locks: repos.device_lock(),
            intents: repos.intent(),
            event_bus,
        }
    }

    pub async fn guard(
        &self,
        origin: CommandOrigin,
        device_id: &str,
    ) -> Result<Option<DeviceLock>, sqlx::Error> {
        match origin {
            CommandOrigin::User => Ok(None),
            CommandOrigin::Workflow => self.locks.active(device_id).await,
        }
    }

    pub async fn lock(
        &self,
        device_id: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<DeviceLock, sqlx::Error> {
        let lock = self
            .locks
            .replace(device_id, DeviceLockSource::Manual, expires_at)
            .await?;

        tracing::info!("{device_id} is locked ({})", describe(&lock));
        self.publish(device_id, Some(&lock));

        Ok(lock)
    }

    pub async fn unlock(&self, device_id: &str) -> Result<bool, sqlx::Error> {
        let removed = self.locks.remove(device_id).await?;

        if removed {
            tracing::info!("{device_id} is unlocked");
            self.publish(device_id, None);
        } else {
            tracing::debug!("{device_id} had no lock to remove");
        }

        Ok(removed)
    }

    pub async fn observe(
        &self,
        device: &LockedDevice<'_>,
        previous: Option<&DeviceReport<'_>>,
        current: &DeviceReport<'_>,
    ) -> Attribution {
        let LockedDevice {
            device_id,
            address,
            settings,
        } = *device;

        let attribution = intent::observe(self.intents, address, previous, current).await;

        let Attribution::External { fields } = &attribution else {
            return attribution;
        };

        let locking = fields
            .iter()
            .filter(|field| !settings.excluded.contains(field))
            .collect::<Vec<_>>();

        if locking.is_empty() {
            tracing::debug!("{device_id} changed {fields:?} outside the gateway, all excluded");
            return attribution;
        }

        tracing::info!("{device_id} changed {locking:?} outside the gateway, locking it");

        if let Err(e) = self
            .intents
            .supersede_pending(current.kind(), address)
            .await
        {
            tracing::warn!("failed to supersede pending intents for {device_id}: {e}");
        }

        let expires_at = Utc::now() + settings.physical;

        match self
            .locks
            .extend(device_id, DeviceLockSource::Physical, expires_at)
            .await
        {
            Ok(Some(lock)) => self.publish(device_id, Some(&lock)),
            Ok(None) => tracing::debug!("{device_id} is already locked for longer"),
            Err(e) => tracing::error!("failed to lock {device_id}: {e}"),
        }

        attribution
    }

    fn publish(&self, device_id: &str, lock: Option<&DeviceLock>) {
        self.event_bus.publish(EventBusMessage::DeviceLock {
            event_id: Uuid::new_v4(),
            device_id: device_id.to_owned(),
            locked: lock.is_some(),
            source: lock.map(|lock| lock.source),
            expires_at: lock.and_then(|lock| lock.expires_at),
        });
    }
}

pub fn describe(lock: &DeviceLock) -> String {
    match lock.expires_at {
        Some(expires_at) => format!("{}, until {}", lock.source, expires_at.to_rfc3339()),
        None => format!("{}, until unlocked", lock.source),
    }
}
