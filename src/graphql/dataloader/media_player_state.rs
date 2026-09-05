use crate::repo::MediaPlayerRepo;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

pub use crate::repo::media_player::MediaPlayerStateRow as MediaPlayerStateModel;

pub struct MediaPlayerStateDataLoader {
    pub repo: MediaPlayerRepo,
}

impl Loader<String> for MediaPlayerStateDataLoader {
    type Value = MediaPlayerStateModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = self.repo.latest_many(keys).await?;

        Ok(rows.into_iter().map(|r| (r.device_id.clone(), r)).collect())
    }
}
