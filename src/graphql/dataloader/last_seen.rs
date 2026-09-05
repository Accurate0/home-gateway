use crate::repo::DeviceRepo;
use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use std::{collections::HashMap, sync::Arc};

pub struct LastSeenDataLoader {
    pub repo: DeviceRepo,
}

impl Loader<String> for LastSeenDataLoader {
    type Value = DateTime<Utc>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = self.repo.last_seen_many(keys).await?;

        Ok(rows.into_iter().map(|r| (r.address, r.last_seen)).collect())
    }
}
