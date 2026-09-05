use crate::actors::integrations::jellyfin::reconcile::Playing;
use crate::event_bus::playback::PlaybackState;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct JellyfinRepo {
    db: Pool<Postgres>,
}

pub struct JellyfinSessionRow {
    pub event_id: Uuid,
    pub session_id: String,
    pub user_name: String,
    pub device_name: String,
    pub client: String,
    pub item_id: String,
    pub item_name: String,
    pub item_type: String,
    pub series_name: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub position_seconds: Option<f64>,
    pub runtime_seconds: Option<f64>,
    pub play_method: Option<String>,
    pub paused: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl JellyfinRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn append_event(
        &self,
        event_id: Uuid,
        playback: PlaybackState,
        playing: &Playing,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO jellyfin_playback_events (
                event_id, state, session_id, user_name, device_name, client,
                item_id, item_name, item_type, series_name, season, episode,
                position_seconds, runtime_seconds, play_method
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"#,
            event_id,
            playback.as_str(),
            playing.session_id,
            playing.user,
            playing.device,
            playing.client,
            playing.item_id,
            playing.item_name,
            playing.item_type,
            playing.series_name,
            playing.season,
            playing.episode,
            playing.position_seconds,
            playing.runtime_seconds,
            playing.play_method,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn delete_session(
        &self,
        user_name: &str,
        session_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"DELETE FROM jellyfin_state WHERE user_name = $1 AND session_id = $2"#,
            user_name,
            session_id,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn clear_sessions(&self) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM jellyfin_state")
            .execute(&self.db)
            .await?;

        Ok(())
    }

    pub async fn upsert_latest(
        &self,
        event_id: Uuid,
        playing: &Playing,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO jellyfin_state (
                user_name, session_id, device_name, client, item_id, item_name,
                item_type, series_name, season, episode, position_seconds,
                runtime_seconds, play_method, paused, event_id, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, now())
            ON CONFLICT (user_name)
            DO UPDATE SET
                session_id = EXCLUDED.session_id,
                device_name = EXCLUDED.device_name,
                client = EXCLUDED.client,
                item_id = EXCLUDED.item_id,
                item_name = EXCLUDED.item_name,
                item_type = EXCLUDED.item_type,
                series_name = EXCLUDED.series_name,
                season = EXCLUDED.season,
                episode = EXCLUDED.episode,
                position_seconds = EXCLUDED.position_seconds,
                runtime_seconds = EXCLUDED.runtime_seconds,
                play_method = EXCLUDED.play_method,
                paused = EXCLUDED.paused,
                event_id = EXCLUDED.event_id,
                updated_at = EXCLUDED.updated_at
            "#,
            playing.user,
            playing.session_id,
            playing.device,
            playing.client,
            playing.item_id,
            playing.item_name,
            playing.item_type,
            playing.series_name,
            playing.season,
            playing.episode,
            playing.position_seconds,
            playing.runtime_seconds,
            playing.play_method,
            playing.paused,
            event_id,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn refresh_latest(&self, playing: &Playing) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE jellyfin_state
            SET position_seconds = $3, paused = $4, updated_at = now()
            WHERE user_name = $1 AND session_id = $2"#,
            playing.user,
            playing.session_id,
            playing.position_seconds,
            playing.paused,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
    pub async fn sessions(&self) -> Result<Vec<JellyfinSessionRow>, sqlx::Error> {
        sqlx::query_as!(
            JellyfinSessionRow,
            r#"SELECT event_id, session_id, user_name, device_name, client, item_id, item_name,
                      item_type, series_name, season, episode, position_seconds, runtime_seconds,
                      play_method, paused, updated_at
               FROM jellyfin_state
               ORDER BY updated_at DESC"#
        )
        .fetch_all(&self.db)
        .await
    }
}
