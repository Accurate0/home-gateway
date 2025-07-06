use crate::types::db::ApplianceStateType;
use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct ApplianceEvent {
    pub time: DateTime<Utc>,
    pub id: Uuid,
    pub entity_id: String,
    pub state: ApplianceStateType,
    pub name: String,
}
