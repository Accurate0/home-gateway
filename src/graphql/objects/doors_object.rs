use crate::types::db::DoorState;
use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct DoorEvent {
    pub time: DateTime<Utc>,
    pub state: DoorState,
    pub name: String,
    pub id: Uuid,
    pub entity_id: String,
}
