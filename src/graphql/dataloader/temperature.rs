use crate::repo::EnvironmentRepo;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

pub use crate::repo::environment::LatestTemperatureRow as TemperatureModel;

pub struct LatestTemperatureDataLoader {
    pub repo: EnvironmentRepo,
}

impl Loader<String> for LatestTemperatureDataLoader {
    type Value = TemperatureModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = self.repo.latest_many(keys).await?;

        Ok(rows.into_iter().map(|r| (r.entity_id.clone(), r)).collect())
    }
}
