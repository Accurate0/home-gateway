use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres, Transaction};
use std::collections::HashMap;

pub struct LedgerRow {
    pub name: String,
    pub checksum: String,
}

pub struct TaskRunRow {
    pub checksum: String,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: i64,
}

pub struct CronRunRow {
    pub last_run_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub rows_affected: i64,
    pub outcome: String,
}

#[derive(Clone)]
pub struct AdhocRepo {
    db: Pool<Postgres>,
}

impl AdhocRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn read_ledger(&self, ordinal: i64) -> Result<Option<LedgerRow>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT name, checksum FROM adhoc_task_run WHERE ordinal = $1",
            ordinal
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| LedgerRow {
            name: row.name,
            checksum: row.checksum,
        }))
    }

    pub async fn write_ledger(
        &self,
        tx: &mut Transaction<'static, Postgres>,
        ordinal: i64,
        name: &str,
        checksum: &str,
        duration_ms: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO adhoc_task_run (ordinal, name, checksum, duration_ms) VALUES ($1, $2, $3, $4)",
            ordinal,
            name,
            checksum,
            duration_ms
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn record_cron_run(
        &self,
        name: &str,
        duration_ms: i64,
        rows_affected: i64,
        outcome: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO adhoc_cron_run (name, last_run_at, duration_ms, rows_affected, outcome) \
             VALUES ($1, now(), $2, $3, $4) \
             ON CONFLICT (name) DO UPDATE SET \
             last_run_at = EXCLUDED.last_run_at, \
             duration_ms = EXCLUDED.duration_ms, \
             rows_affected = EXCLUDED.rows_affected, \
             outcome = EXCLUDED.outcome",
            name,
            duration_ms,
            rows_affected,
            outcome
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn task_runs(&self) -> Result<HashMap<i64, TaskRunRow>, sqlx::Error> {
        let rows =
            sqlx::query!("SELECT ordinal, checksum, completed_at, duration_ms FROM adhoc_task_run")
                .fetch_all(&self.db)
                .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.ordinal,
                    TaskRunRow {
                        checksum: row.checksum,
                        completed_at: row.completed_at,
                        duration_ms: row.duration_ms,
                    },
                )
            })
            .collect())
    }

    pub async fn cron_runs(&self) -> Result<HashMap<String, CronRunRow>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT name, last_run_at, duration_ms, rows_affected, outcome FROM adhoc_cron_run"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.name,
                    CronRunRow {
                        last_run_at: row.last_run_at,
                        duration_ms: row.duration_ms,
                        rows_affected: row.rows_affected,
                        outcome: row.outcome,
                    },
                )
            })
            .collect())
    }
}
