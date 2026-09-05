use crate::repo::HomeAssistantRepo;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

pub use crate::repo::home_assistant::HomeAssistantStateRow as HomeAssistantStateModel;

pub struct HomeAssistantStateDataLoader {
    pub repo: HomeAssistantRepo,
}

impl Loader<String> for HomeAssistantStateDataLoader {
    type Value = HomeAssistantStateModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = self.repo.latest_many(keys).await?;

        Ok(rows.into_iter().map(|r| (r.entity_id.clone(), r)).collect())
    }
}
