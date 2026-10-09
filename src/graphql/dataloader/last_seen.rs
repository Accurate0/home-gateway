use crate::device_registry::last_seen::LastSeen;
use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use std::{collections::HashMap, sync::Arc};

pub struct LastSeenDataLoader {
    pub last_seen: LastSeen,
}

impl Loader<String> for LastSeenDataLoader {
    type Value = DateTime<Utc>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        Ok(self.last_seen.lookup(keys).await?)
    }
}
