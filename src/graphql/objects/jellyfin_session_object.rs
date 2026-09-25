use async_graphql::{ComplexObject, ID, SimpleObject};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::repo::jellyfin::JellyfinSessionRow;

#[derive(SimpleObject)]
#[graphql(name = "JellyfinSession", complex, rename_fields = "camelCase")]
pub struct JellyfinSessionObject {
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
impl JellyfinSessionObject {
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

impl From<JellyfinSessionRow> for JellyfinSessionObject {
    fn from(row: JellyfinSessionRow) -> Self {
        Self {
            event_id: row.event_id,
            session_id: row.session_id,
            user: row.user_name,
            device: row.device_name,
            client: row.client,
            item_id: row.item_id,
            item_name: row.item_name,
            item_type: row.item_type,
            series_name: row.series_name,
            season: row.season,
            episode: row.episode,
            position_seconds: row.position_seconds,
            runtime_seconds: row.runtime_seconds,
            play_method: row.play_method,
            paused: row.paused,
            updated_at: row.updated_at,
        }
    }
}
