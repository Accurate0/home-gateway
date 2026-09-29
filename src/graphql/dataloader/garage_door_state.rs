use crate::repo::GarageDoorRepo;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

pub use crate::repo::garage_door::GarageDoorStateRow as GarageDoorStateModel;

pub struct GarageDoorStateDataLoader {
    pub repo: GarageDoorRepo,
}

impl Loader<String> for GarageDoorStateDataLoader {
    type Value = GarageDoorStateModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = self.repo.latest_many(keys).await?;

        Ok(rows.into_iter().map(|r| (r.device_id.clone(), r)).collect())
    }
}
