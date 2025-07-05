use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct DoorEvent {
    pub time: DateTime<Utc>,
    pub contact: bool,
    pub name: String,
}
