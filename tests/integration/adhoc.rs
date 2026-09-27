use home_gateway::adhoc::AdhocTaskContext;
use home_gateway::adhoc::runner::run_pending;
use home_gateway::adhoc::tasks::rekey_device_ids::rekey_all;
use home_gateway::repo::AdhocRepo;
use pretty_assertions::assert_eq;
use serial_test::serial;
use sqlx::{Pool, Postgres};

use crate::common::Harness;
use crate::common::db::fresh_database;

const ORDINAL: i64 = 20260816120000;

#[tokio::test]
async fn ledger_round_trips() {
    let pool = fresh_database().await.pool;

    assert!(
        AdhocRepo::new(pool.clone())
            .read_ledger(ORDINAL)
            .await
            .unwrap()
            .is_none()
    );

    let mut tx = pool.begin().await.unwrap();
    AdhocRepo::new(pool.clone())
        .write_ledger(&mut tx, ORDINAL, "trim", "abc", 12)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let row = AdhocRepo::new(pool.clone())
        .read_ledger(ORDINAL)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(row.name, "trim");
    assert_eq!(row.checksum, "abc");
}

#[tokio::test]
async fn rolled_back_ledger_leaves_task_pending() {
    let pool = fresh_database().await.pool;

    let mut tx = pool.begin().await.unwrap();
    AdhocRepo::new(pool.clone())
        .write_ledger(&mut tx, ORDINAL, "trim", "abc", 12)
        .await
        .unwrap();
    tx.rollback().await.unwrap();

    assert!(
        AdhocRepo::new(pool.clone())
            .read_ledger(ORDINAL)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn ledger_rejects_a_second_run_of_the_same_ordinal() {
    let pool = fresh_database().await.pool;

    let mut tx = pool.begin().await.unwrap();
    AdhocRepo::new(pool.clone())
        .write_ledger(&mut tx, ORDINAL, "trim", "abc", 12)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    let second = AdhocRepo::new(pool.clone())
        .write_ledger(&mut tx, ORDINAL, "trim", "abc", 12)
        .await;

    assert!(second.is_err());
}

#[tokio::test]
async fn cron_run_upserts_by_name() {
    let pool = fresh_database().await.pool;

    AdhocRepo::new(pool.clone())
        .record_cron_run("trim", 10, 5, "success")
        .await
        .unwrap();
    AdhocRepo::new(pool.clone())
        .record_cron_run("trim", 20, 7, "error")
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

const REKEY_ORDINAL: i64 = 3;

async fn count(db: &Pool<Postgres>, sql: &str, key: &str) -> i64 {
    sqlx::query_scalar(sql)
        .bind(key)
        .fetch_one(db)
        .await
        .unwrap()
}

async fn seed_old_ids(db: &Pool<Postgres>) {
    for minutes in 0..5 {
        sqlx::query(
            "INSERT INTO temperature_sensor (event_id, name, ieee_addr, temperature, id, \"time\") \
             VALUES (gen_random_uuid(), 'Bedroom', '0xb0c7defffef6a9be', 21, 'bedroom', \
                     now() - make_interval(mins => $1))",
        )
        .bind(minutes)
        .execute(db)
        .await
        .unwrap();
    }

    for minutes in 0..3 {
        sqlx::query(
            "INSERT INTO device_battery (event_id, device_id, kind, battery_percent, \"time\") \
             VALUES (gen_random_uuid(), 'front-door', 'percent', 90, now() - make_interval(mins => $1))",
        )
        .bind(minutes)
        .execute(db)
        .await
        .unwrap();
    }

    sqlx::query(
        "INSERT INTO latest_temperature_sensor (name, entity_id, ieee_addr, temperature) \
         VALUES ('Bedroom', 'bedroom', '0xb0c7defffef6a9be', 21)",
    )
    .execute(db)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO device_battery_latest (device_id, name, kind, battery_percent) \
         VALUES ('front-door', 'front-door', 'percent', 80), \
                ('door-mccgq12lm-1', 'door-mccgq12lm-1', 'percent', 90)",
    )
    .execute(db)
    .await
    .unwrap();

    sqlx::query("INSERT INTO device_last_seen (device_key) VALUES ('zigbee:front-door')")
        .execute(db)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
async fn rekey_commits_each_batch_so_progress_survives_a_rolled_back_run() {
    let harness = Harness::start().await;
    let db = &harness.db;

    seed_old_ids(db).await;

    let mut tx = db.begin().await.unwrap();
    {
        let mut ctx = AdhocTaskContext::new(&harness.state, &mut tx);
        rekey_all(&mut ctx, 2).await.unwrap();
    }
    tx.rollback().await.unwrap();

    let temperatures = "SELECT count(*) FROM temperature_sensor WHERE id = $1";
    assert_eq!(count(db, temperatures, "bedroom").await, 0);
    assert_eq!(count(db, temperatures, "env-vindstyrka-1").await, 5);

    let batteries = "SELECT count(*) FROM device_battery WHERE device_id = $1";
    assert_eq!(count(db, batteries, "front-door").await, 0);
    assert_eq!(count(db, batteries, "door-mccgq12lm-1").await, 3);

    let latest_temperature = "SELECT count(*) FROM latest_temperature_sensor WHERE entity_id = $1";
    assert_eq!(count(db, latest_temperature, "env-vindstyrka-1").await, 1);

    let latest_battery: Vec<(String, f64)> = sqlx::query_as(
        "SELECT device_id, battery_percent FROM device_battery_latest ORDER BY device_id",
    )
    .fetch_all(db)
    .await
    .unwrap();
    assert_eq!(latest_battery, [("door-mccgq12lm-1".to_owned(), 90.0)]);

    let last_seen = "SELECT count(*) FROM device_last_seen WHERE device_key = $1";
    assert_eq!(count(db, last_seen, "zigbee:door-mccgq12lm-1").await, 1);
    assert_eq!(count(db, last_seen, "zigbee:front-door").await, 0);
}

#[tokio::test]
#[serial]
async fn rekey_resumes_after_a_partial_run_and_records_its_ledger_once() {
    let harness = Harness::start().await;
    let db = &harness.db;

    seed_old_ids(db).await;

    sqlx::query(
        "UPDATE temperature_sensor SET id = 'env-vindstyrka-1' WHERE id = 'bedroom' \
         AND \"time\" > now() - interval '2 minutes'",
    )
    .execute(db)
    .await
    .unwrap();

    run_pending(&harness.state, true).await;

    let temperatures = "SELECT count(*) FROM temperature_sensor WHERE id = $1";
    assert_eq!(count(db, temperatures, "bedroom").await, 0);
    assert_eq!(count(db, temperatures, "env-vindstyrka-1").await, 5);

    let ledger = AdhocRepo::new(db.clone())
        .read_ledger(REKEY_ORDINAL)
        .await
        .unwrap()
        .expect("the rekey task wrote its ledger row");
    assert_eq!(ledger.name, "rekey_device_ids");

    run_pending(&harness.state, true).await;

    let runs: i64 = sqlx::query_scalar("SELECT count(*) FROM adhoc_task_run WHERE ordinal = $1")
        .bind(REKEY_ORDINAL)
        .fetch_one(db)
        .await
        .unwrap();
    assert_eq!(runs, 1);
}
