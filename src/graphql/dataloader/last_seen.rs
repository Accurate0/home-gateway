use crate::device_registry::{DeviceRegistry, last_seen};
use crate::repo::DeviceRepo;
use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use std::{collections::HashMap, sync::Arc};

pub struct LastSeenDataLoader {
    pub repo: DeviceRepo,
    pub devices: DeviceRegistry,
}

impl Loader<String> for LastSeenDataLoader {
    type Value = DateTime<Utc>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        Ok(last_seen::lookup(&self.devices, &self.repo, keys).await?)
    }
}
