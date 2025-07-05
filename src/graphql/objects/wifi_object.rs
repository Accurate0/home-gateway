use crate::types::db::UnifiState;
use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct WifiEvent {
    pub name: String,
    pub id: String,
    pub time: DateTime<Utc>,
    pub state: UnifiState,
}
