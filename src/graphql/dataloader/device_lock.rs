use crate::repo::DeviceLockRepo;
use crate::repo::device_lock::DeviceLock;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

pub struct DeviceLockDataLoader {
    pub repo: DeviceLockRepo,
}

impl Loader<String> for DeviceLockDataLoader {
    type Value = DeviceLock;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = self.repo.active_many(keys).await?;

        Ok(rows
            .into_iter()
            .map(|lock| (lock.device_id.clone(), lock))
            .collect())
    }
}
