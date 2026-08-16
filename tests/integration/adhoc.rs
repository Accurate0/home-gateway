use home_gateway::adhoc::queries::{read_ledger, record_cron_run, write_ledger};
use pretty_assertions::assert_eq;

use crate::common::db::fresh_database;

const ORDINAL: i64 = 20260816120000;

#[tokio::test]
async fn ledger_round_trips() {
    let pool = fresh_database().await.pool;

    assert!(read_ledger(&pool, ORDINAL).await.unwrap().is_none());

    let mut tx = pool.begin().await.unwrap();
    write_ledger(&mut tx, ORDINAL, "trim", "abc", 12)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let row = read_ledger(&pool, ORDINAL).await.unwrap().unwrap();

    assert_eq!(row.name, "trim");
    assert_eq!(row.checksum, "abc");
}

#[tokio::test]
async fn rolled_back_ledger_leaves_task_pending() {
    let pool = fresh_database().await.pool;

    let mut tx = pool.begin().await.unwrap();
    write_ledger(&mut tx, ORDINAL, "trim", "abc", 12)
        .await
        .unwrap();
    tx.rollback().await.unwrap();

    assert!(read_ledger(&pool, ORDINAL).await.unwrap().is_none());
}

#[tokio::test]
async fn ledger_rejects_a_second_run_of_the_same_ordinal() {
    let pool = fresh_database().await.pool;

    let mut tx = pool.begin().await.unwrap();
    write_ledger(&mut tx, ORDINAL, "trim", "abc", 12)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    let second = write_ledger(&mut tx, ORDINAL, "trim", "abc", 12).await;

    assert!(second.is_err());
}

#[tokio::test]
async fn cron_run_upserts_by_name() {
    let pool = fresh_database().await.pool;

    record_cron_run(&pool, "trim", 10, 5, "success")
        .await
        .unwrap();
    record_cron_run(&pool, "trim", 20, 7, "error")
        .await
        .unwrap();

    let rows: Vec<(i64, String)> =
        sqlx::query_as("SELECT rows_affected, outcome FROM adhoc_cron_run WHERE name = $1")
            .bind("trim")
            .fetch_all(&pool)
            .await
            .unwrap();

    assert_eq!(
        rows.len(),
        1,
        "the upsert should keep a single row per task"
    );
    assert_eq!(rows[0].0, 7);
    assert_eq!(rows[0].1, "error");
}
