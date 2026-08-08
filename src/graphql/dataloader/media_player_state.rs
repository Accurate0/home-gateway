use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tracing::Instrument;

pub struct MediaPlayerStateDataLoader {
    pub database: Pool<Postgres>,
}

#[derive(Clone)]
pub struct MediaPlayerStateModel {
    pub device_id: String,
    pub state: String,
    pub app_name: Option<String>,
    pub source: Option<String>,
    pub media_content_type: Option<String>,
    pub media_title: Option<String>,
    pub media_artist: Option<String>,
    pub media_album_name: Option<String>,
    pub media_series_title: Option<String>,
    pub media_season: Option<i32>,
    pub media_episode: Option<i32>,
    pub position_seconds: Option<f64>,
    pub duration_seconds: Option<f64>,
    pub position_updated_at: Option<DateTime<Utc>>,
    pub volume_level: Option<f64>,
    pub muted: Option<bool>,
    pub artwork_url: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl Loader<String> for MediaPlayerStateDataLoader {
    type Value = MediaPlayerStateModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = sqlx::query_as!(
            MediaPlayerStateModel,
            r#"
            SELECT device_id, state, app_name, source, media_content_type, media_title,
                   media_artist, media_album_name, media_series_title, media_season, media_episode,
                   position_seconds, duration_seconds, position_updated_at, volume_level, muted,
                   artwork_url, updated_at
            FROM media_player_state
            WHERE device_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.database)
        .instrument(tracing::info_span!("bulk-get-media-player-state"))
        .await?;

        Ok(rows.into_iter().map(|r| (r.device_id.clone(), r)).collect())
    }
}
