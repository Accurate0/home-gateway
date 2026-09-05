use crate::actors::devices::media_player::state::{Attributes, Prior};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use tracing::Instrument;
use uuid::Uuid;

pub struct MediaPlayerUpdate<'a> {
    pub event_id: Uuid,
    pub device_id: &'a str,
    pub state: &'a str,
    pub attributes: &'a Attributes,
    pub artwork_url: Option<&'a str>,
    pub raw_attributes: &'a serde_json::Value,
}

#[derive(Clone)]
pub struct MediaPlayerRepo {
    db: Pool<Postgres>,
}

#[derive(Clone)]
pub struct MediaPlayerStateRow {
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

impl MediaPlayerRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn load_prior(&self, device_id: &str) -> Result<Option<Prior>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT state, media_title FROM media_player_state WHERE device_id = $1",
            device_id,
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| Prior {
            state: row.state,
            media_title: row.media_title,
        }))
    }

    pub async fn upsert(&self, update: &MediaPlayerUpdate<'_>) -> Result<(), sqlx::Error> {
        let attributes = update.attributes;

        sqlx::query!(
            r#"INSERT INTO media_player_state (
                device_id, state, app_name, source, media_content_type, media_title,
                media_artist, media_album_name, media_series_title, media_season, media_episode,
                position_seconds, duration_seconds, position_updated_at, volume_level, muted,
                artwork_url, attributes, event_id, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, now())
            ON CONFLICT (device_id) DO UPDATE SET
                state = EXCLUDED.state,
                app_name = EXCLUDED.app_name,
                source = EXCLUDED.source,
                media_content_type = EXCLUDED.media_content_type,
                media_title = EXCLUDED.media_title,
                media_artist = EXCLUDED.media_artist,
                media_album_name = EXCLUDED.media_album_name,
                media_series_title = EXCLUDED.media_series_title,
                media_season = EXCLUDED.media_season,
                media_episode = EXCLUDED.media_episode,
                position_seconds = EXCLUDED.position_seconds,
                duration_seconds = EXCLUDED.duration_seconds,
                position_updated_at = EXCLUDED.position_updated_at,
                volume_level = COALESCE(EXCLUDED.volume_level, media_player_state.volume_level),
                muted = COALESCE(EXCLUDED.muted, media_player_state.muted),
                artwork_url = EXCLUDED.artwork_url,
                attributes = EXCLUDED.attributes,
                event_id = EXCLUDED.event_id,
                updated_at = now()
            "#,
            update.device_id,
            update.state,
            attributes.app_name,
            attributes.source,
            attributes.media_content_type,
            attributes.media_title,
            attributes.media_artist,
            attributes.media_album_name,
            attributes.media_series_title,
            attributes.media_season,
            attributes.media_episode,
            attributes.media_position,
            attributes.media_duration,
            attributes.media_position_updated_at,
            attributes.volume_level,
            attributes.is_volume_muted,
            update.artwork_url,
            update.raw_attributes,
            update.event_id,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
    pub async fn latest_many(
        &self,
        keys: &[String],
    ) -> Result<Vec<MediaPlayerStateRow>, sqlx::Error> {
        sqlx::query_as!(
            MediaPlayerStateRow,
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
        .fetch_all(&self.db)
        .instrument(tracing::info_span!("bulk-get-media-player-state"))
        .await
    }
}
