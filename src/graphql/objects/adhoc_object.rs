use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct AdhocCronTaskStatus {
    pub id: String,
    pub name: String,
    pub schedule: String,
    pub flag: Option<String>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub rows_affected: Option<i64>,
    pub outcome: Option<String>,
}

#[derive(SimpleObject)]
pub struct AdhocTaskStatus {
    pub id: String,
    pub ordinal: i64,
    pub name: String,
    pub flag: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub pending: bool,
    pub checksum_drifted: bool,
}
