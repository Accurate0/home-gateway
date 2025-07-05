use crate::types::db::ApplianceStateType;
use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct ApplianceEvent {
    pub time: DateTime<Utc>,
    pub id: String,
    pub state: ApplianceStateType,
    pub name: String,
}
