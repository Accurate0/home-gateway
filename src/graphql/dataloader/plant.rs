use crate::repo::PlantRepo;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

pub use crate::repo::plant::LatestPlantRow as PlantModel;

pub struct LatestPlantDataLoader {
    pub repo: PlantRepo,
}

impl Loader<String> for LatestPlantDataLoader {
    type Value = PlantModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = self.repo.latest_many(keys).await?;

        Ok(rows.into_iter().map(|r| (r.entity_id.clone(), r)).collect())
    }
}
