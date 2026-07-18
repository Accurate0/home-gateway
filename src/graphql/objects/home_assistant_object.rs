use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct HomeAssistantEvent {
    pub event_id: Uuid,
    pub entity_id: String,
    pub state: String,
    pub time: DateTime<Utc>,
}
