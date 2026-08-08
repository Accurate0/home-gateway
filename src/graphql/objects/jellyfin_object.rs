use async_graphql::{ComplexObject, ID, InputObject, Object, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(SimpleObject)]
#[graphql(complex, rename_fields = "camelCase")]
pub struct JellyfinSession {
    pub event_id: Uuid,
    pub session_id: String,
    pub user: String,
    pub device: String,
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
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl JellyfinSession {
    async fn id(&self) -> ID {
        ID(self.session_id.clone())
    }

    async fn progress(&self) -> Option<f64> {
        match (self.position_seconds, self.runtime_seconds) {
            (Some(position), Some(runtime)) if runtime > 0.0 => Some(position / runtime),
            _ => None,
        }
    }
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct JellyfinPlaybackEvent {
    pub event_id: Uuid,
    pub state: String,
    pub session_id: String,
    pub user: String,
    pub device: String,
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
    pub time: DateTime<Utc>,
}

#[derive(InputObject)]
pub struct JellyfinHistoryInput {
    pub since: DateTime<Utc>,
    pub limit: Option<i64>,
}

pub struct JellyfinObject {}

#[Object]
impl JellyfinObject {
    async fn now_playing(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<JellyfinSession>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(sqlx::query!(
            r#"SELECT event_id, session_id, user_name, device_name, client, item_id, item_name,
                      item_type, series_name, season, episode, position_seconds, runtime_seconds,
                      play_method, paused, updated_at
               FROM latest_jellyfin_session
               WHERE item_id IS NOT NULL
               ORDER BY updated_at DESC"#
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| JellyfinSession {
            event_id: r.event_id,
            session_id: r.session_id,
            user: r.user_name,
            device: r.device_name,
            client: r.client,
            item_id: r.item_id.unwrap_or_default(),
            item_name: r.item_name.unwrap_or_default(),
            item_type: r.item_type.unwrap_or_default(),
            series_name: r.series_name,
            season: r.season,
            episode: r.episode,
            position_seconds: r.position_seconds,
            runtime_seconds: r.runtime_seconds,
            play_method: r.play_method,
            paused: r.paused,
            updated_at: r.updated_at,
        })
        .collect_vec())
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        input: JellyfinHistoryInput,
    ) -> async_graphql::Result<Vec<JellyfinPlaybackEvent>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(sqlx::query!(
            r#"SELECT event_id, state, session_id, user_name, device_name, client, item_id,
                      item_name, item_type, series_name, season, episode, position_seconds,
                      runtime_seconds, play_method, "time"
               FROM jellyfin_playback_events
               WHERE "time" >= $1
               ORDER BY "time" DESC
               LIMIT $2"#,
            input.since,
            input.limit.unwrap_or(100),
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| JellyfinPlaybackEvent {
            event_id: r.event_id,
            state: r.state,
            session_id: r.session_id,
            user: r.user_name,
            device: r.device_name,
            client: r.client,
            item_id: r.item_id,
            item_name: r.item_name,
            item_type: r.item_type,
            series_name: r.series_name,
            season: r.season,
            episode: r.episode,
            position_seconds: r.position_seconds,
            runtime_seconds: r.runtime_seconds,
            play_method: r.play_method,
            time: r.time,
        })
        .collect_vec())
    }
}
