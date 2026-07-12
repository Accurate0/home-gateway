use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct WorkflowRun {
    pub id: async_graphql::ID,
    pub slug: String,
    pub name: String,
    pub event_id: String,
    pub outcome: String,
    pub dry_run: bool,
    pub duration_ms: i64,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
}

#[derive(SimpleObject)]
pub struct WorkflowStatus {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub group: String,
    pub enabled: bool,
    pub config_enabled: bool,
    pub dry_run: bool,
    pub reusable: bool,
}
