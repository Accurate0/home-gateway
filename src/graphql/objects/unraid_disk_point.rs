use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

use crate::repo::unraid::UnraidDiskHistoryRow;

#[derive(SimpleObject)]
pub struct UnraidDiskPoint {
    pub time: DateTime<Utc>,
    pub status: Option<String>,
    pub temperature: Option<i32>,
    pub fs_used_kb: Option<i64>,
    pub fs_size_kb: Option<i64>,
    pub usage_percent: Option<f64>,
    pub errors: Option<i64>,
    pub spinning: Option<bool>,
}

impl From<UnraidDiskHistoryRow> for UnraidDiskPoint {
    fn from(row: UnraidDiskHistoryRow) -> Self {
        let usage_percent = match (row.fs_used_kb, row.fs_size_kb) {
            (Some(used), Some(size)) if size > 0 => Some(used as f64 / size as f64 * 100.0),
            _ => None,
        };

        Self {
            time: row.time,
            status: row.status,
            temperature: row.temperature,
            fs_used_kb: row.fs_used_kb,
            fs_size_kb: row.fs_size_kb,
            usage_percent,
            errors: row.errors,
            spinning: row.spinning,
        }
    }
}
