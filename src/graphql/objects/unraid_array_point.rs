use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

use crate::repo::unraid::UnraidArrayHistoryRow;

#[derive(SimpleObject)]
pub struct UnraidArrayPoint {
    pub time: DateTime<Utc>,
    pub kilobytes_used: i64,
    pub kilobytes_total: i64,
    pub usage_percent: Option<f64>,
    pub parity_status: String,
    pub parity_progress: Option<i64>,
}

impl From<UnraidArrayHistoryRow> for UnraidArrayPoint {
    fn from(row: UnraidArrayHistoryRow) -> Self {
        let usage_percent = (row.kilobytes_total > 0)
            .then(|| row.kilobytes_used as f64 / row.kilobytes_total as f64 * 100.0);

        Self {
            time: row.time,
            kilobytes_used: row.kilobytes_used,
            kilobytes_total: row.kilobytes_total,
            usage_percent,
            parity_status: row.parity_status,
            parity_progress: row.parity_progress,
        }
    }
}
