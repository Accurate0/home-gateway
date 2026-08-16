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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[sqlx::test]
    async fn ledger_round_trips(pool: Pool<Postgres>) {
        assert!(read_ledger(&pool, 20260816120000).await.unwrap().is_none());

        let mut tx = pool.begin().await.unwrap();
        write_ledger(&mut tx, 20260816120000, "trim", "abc", 12)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let row = read_ledger(&pool, 20260816120000).await.unwrap().unwrap();

        assert_eq!(row.name, "trim");
        assert_eq!(row.checksum, "abc");
    }

    #[sqlx::test]
    async fn rolled_back_ledger_leaves_task_pending(pool: Pool<Postgres>) {
        let mut tx = pool.begin().await.unwrap();
        write_ledger(&mut tx, 20260816120000, "trim", "abc", 12)
            .await
            .unwrap();
        tx.rollback().await.unwrap();

        assert!(read_ledger(&pool, 20260816120000).await.unwrap().is_none());
    }

    #[sqlx::test]
    async fn ledger_rejects_a_second_run_of_the_same_ordinal(pool: Pool<Postgres>) {
        let mut tx = pool.begin().await.unwrap();
        write_ledger(&mut tx, 20260816120000, "trim", "abc", 12)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let mut tx = pool.begin().await.unwrap();
        let second = write_ledger(&mut tx, 20260816120000, "trim", "abc", 12).await;

        assert!(second.is_err());
    }

    #[sqlx::test]
    async fn cron_run_upserts_by_name(pool: Pool<Postgres>) {
        record_cron_run(&pool, "trim", 10, 5, "success")
            .await
            .unwrap();
        record_cron_run(&pool, "trim", 20, 7, "error")
            .await
            .unwrap();

        let rows = sqlx::query!(
            "SELECT rows_affected, outcome FROM adhoc_cron_run WHERE name = $1",
            "trim"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].rows_affected, 7);
        assert_eq!(rows[0].outcome, "error");
    }
}
