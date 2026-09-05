use crate::repo::EinkRepo;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

pub use crate::repo::eink::EinkDisplayRow as EinkDisplayModel;

pub struct EinkDisplayDataLoader {
    pub repo: EinkRepo,
}

impl Loader<String> for EinkDisplayDataLoader {
    type Value = EinkDisplayModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = self.repo.battery_many(keys).await?;

        Ok(rows.into_iter().map(|r| (r.device_id.clone(), r)).collect())
    }
}
