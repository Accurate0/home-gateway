use async_graphql::Object;
use chrono::{TimeDelta, Utc};

use crate::device_lock::DeviceLocks;
use crate::event_bus::EventBus;
use crate::graphql::objects::device_lock_object::DeviceLockObject;
use crate::repo::RepoRegistry;

pub struct DeviceLockMutation {
    device_id: String,
}

impl DeviceLockMutation {
    pub fn new(device_id: String) -> Self {
        Self { device_id }
    }
}

#[Object]
impl DeviceLockMutation {
    async fn lock(
        &self,
        ctx: &async_graphql::Context<'_>,
        duration_seconds: Option<u32>,
    ) -> async_graphql::Result<DeviceLockObject> {
        let repos = ctx.data::<RepoRegistry>()?;
        let event_bus = ctx.data::<EventBus>()?;

        let expires_at =
            duration_seconds.map(|seconds| Utc::now() + TimeDelta::seconds(i64::from(seconds)));

        let lock = DeviceLocks::new(repos, event_bus)
            .lock(&self.device_id, expires_at)
            .await?;

        Ok(lock.into())
    }

    async fn unlock(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let repos = ctx.data::<RepoRegistry>()?;
        let event_bus = ctx.data::<EventBus>()?;

        Ok(DeviceLocks::new(repos, event_bus)
            .unlock(&self.device_id)
            .await?)
    }
}
