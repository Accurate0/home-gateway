use crate::types::db::DoorState;
use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct DoorEvent {
    pub time: DateTime<Utc>,
    pub state: DoorState,
    pub name: String,
    pub id: String,
}
