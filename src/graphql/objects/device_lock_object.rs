use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

use crate::repo::device_lock::{DeviceLock, DeviceLockSource};

#[derive(SimpleObject)]
#[graphql(name = "DeviceLock")]
pub struct DeviceLockObject {
    pub source: DeviceLockSource,
    pub locked_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<DeviceLock> for DeviceLockObject {
    fn from(lock: DeviceLock) -> Self {
        Self {
            source: lock.source,
            locked_at: lock.locked_at,
            expires_at: lock.expires_at,
        }
    }
}
