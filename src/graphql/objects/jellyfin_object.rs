use crate::repo::RepoRegistry;
use async_graphql::{ComplexObject, ID, Object, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;
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

pub struct JellyfinObject {}

#[Object]
impl JellyfinObject {
    async fn now_playing(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<JellyfinSession>> {
        let repos = ctx.data::<RepoRegistry>()?;

        Ok(repos
            .jellyfin()
            .sessions()
            .await?
            .into_iter()
            .map(|r| JellyfinSession {
                event_id: r.event_id,
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
                paused: r.paused,
                updated_at: r.updated_at,
            })
            .collect_vec())
    }
}
