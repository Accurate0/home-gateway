use async_graphql::{ComplexObject, SimpleObject, ID};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct HomeAssistantEvent {
    pub event_id: Uuid,
    pub entity_id: String,
    pub state: String,
    pub time: DateTime<Utc>,
}

#[ComplexObject]
impl HomeAssistantEvent {
    async fn id(&self) -> ID {
        ID(self.event_id.to_string())
    }
}
