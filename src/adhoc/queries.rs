use sqlx::{Pool, Postgres, Transaction};

pub struct LedgerRow {
    pub name: String,
    pub checksum: String,
}

pub async fn read_ledger(
    db: &Pool<Postgres>,
    ordinal: i64,
) -> Result<Option<LedgerRow>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT name, checksum FROM adhoc_task_run WHERE ordinal = $1",
        ordinal
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|row| LedgerRow {
        name: row.name,
        checksum: row.checksum,
    }))
}

pub async fn write_ledger(
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
    db: &Pool<Postgres>,
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
    .execute(db)
    .await?;

    Ok(())
}
