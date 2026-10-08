use crate::repo::AirPurifierRepo;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

pub use crate::repo::air_purifier::AirPurifierStateRow as AirPurifierStateModel;

pub struct AirPurifierStateDataLoader {
    pub repo: AirPurifierRepo,
}

impl Loader<String> for AirPurifierStateDataLoader {
    type Value = AirPurifierStateModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = self.repo.latest_many(keys).await?;

        Ok(rows.into_iter().map(|r| (r.device_id.clone(), r)).collect())
    }
}
