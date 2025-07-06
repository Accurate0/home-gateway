use crate::types::db::UnifiState;
use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct WifiEvent {
    pub name: String,
    pub id: Uuid,
    pub time: DateTime<Utc>,
    pub state: UnifiState,
}
