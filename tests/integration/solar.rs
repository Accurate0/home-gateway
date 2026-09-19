use chrono::{DateTime, TimeZone, Utc};
use home_gateway::integrations::solar::queries::{
    SolarQueryError, current, history_since, statistics,
};
use pretty_assertions::assert_eq;
use sqlx::{Pool, Postgres};

use crate::common::db::fresh_database;

async fn insert(db: &Pool<Postgres>, at: DateTime<Utc>, kwh: f64, uv: Option<f64>) {
    sqlx::query(
        "INSERT INTO solar_data_tsdb \
             (current_kwh, today_kwh, month_kwh, total_kwh, raw_data, uv_level, temperature, time) \
         VALUES ($1, $2, 1.0, 3.0, $3, $4, $5, $6)",
    )
    .bind(kwh)
    .bind(kwh / 100.0)
    .bind(serde_json::json!({"data": {"kpi": {
        "month_generation": 1.0, "pac": kwh, "power": 2.0, "total_power": 3.0,
        "day_income": 0.0, "total_income": 0.0, "yield_rate": 0.0, "currency": "AUD"
    }}}))
    .bind(uv)
    .bind(Some(20.0_f64))
    .bind(at)
    .execute(db)
    .await
    .unwrap();
}

#[tokio::test]
async fn history_buckets_align_to_five_minute_utc_boundaries() {
    let db = fresh_database().await.pool;

    let yesterday = (Utc::now() - chrono::Duration::days(1)).timestamp();
    let bucket = Utc
        .timestamp_opt(yesterday - yesterday.rem_euclid(300), 0)
        .unwrap();
    let base = bucket + chrono::Duration::seconds(150);
    insert(&db, base, 100.0, Some(5.0)).await;
    insert(&db, base + chrono::Duration::minutes(1), 200.0, Some(7.0)).await;

    let history = history_since(&db, base - chrono::Duration::hours(1))
        .await
        .unwrap();

    assert_eq!(history.len(), 1);
    let point = &history[0];

    assert_eq!(point.at, bucket.naive_utc());
    assert_eq!(point.wh, 150.0);
    assert_eq!(point.uv_level, Some(6.0));
    assert_eq!(point.timestamp, point.at.and_utc().timestamp_millis());
}

#[tokio::test]
async fn history_is_clamped_to_the_window() {
    let db = fresh_database().await.pool;

    let now = Utc::now();
    insert(&db, now - chrono::Duration::days(60), 500.0, None).await;
    insert(&db, now - chrono::Duration::days(1), 100.0, None).await;

    let history = history_since(&db, now - chrono::Duration::days(365))
        .await
        .unwrap();

    assert_eq!(
        history.len(),
        1,
        "a since older than the window must be clamped"
    );
    assert_eq!(history[0].wh, 100.0);
}

#[tokio::test]
async fn averages_ignore_rows_outside_the_window() {
    let db = fresh_database().await.pool;

    let now = Utc::now();
    insert(&db, now - chrono::Duration::minutes(5), 100.0, None).await;
    insert(&db, now - chrono::Duration::minutes(45), 300.0, None).await;

    let stats = statistics(&db).await.unwrap();

    assert_eq!(stats.averages.last_15_mins, Some(100.0));
    assert_eq!(stats.averages.last_1_hour, Some(200.0));
    assert_eq!(stats.averages.last_3_hours, Some(200.0));
}

#[tokio::test]
async fn current_reports_no_data_on_an_empty_table() {
    let db = fresh_database().await.pool;

    assert!(matches!(current(&db).await, Err(SolarQueryError::NoData)));
}

#[tokio::test]
async fn current_falls_back_to_raw_data_for_rows_without_kpi_columns() {
    let db = fresh_database().await.pool;

    sqlx::query("INSERT INTO solar_data_tsdb (current_kwh, raw_data, time) VALUES ($1, $2, $3)")
        .bind(250.0_f64)
        .bind(serde_json::json!({"data": {"kpi": {
            "month_generation": 7.0, "pac": 250.0, "power": 5.0, "total_power": 9.0
        }}}))
        .bind(Utc::now() - chrono::Duration::minutes(1))
        .execute(&db)
        .await
        .unwrap();

    let solar = current(&db).await.unwrap();

    assert_eq!(solar.today_production_kwh, 5.0);
    assert_eq!(solar.month_production_kwh, 7.0);
    assert_eq!(solar.all_time_production_kwh, 9.0);
}

#[tokio::test]
async fn backfill_fills_compressed_legacy_rows_in_batches() {
    let db = fresh_database().await.pool;

    let start = Utc::now() - chrono::Duration::days(30);

    for n in 0..5 {
        sqlx::query(
            "INSERT INTO solar_data_tsdb (current_kwh, raw_data, time) VALUES ($1, $2, $3)",
        )
        .bind(100.0_f64)
        .bind(serde_json::json!({"data": {"kpi": {
            "month_generation": 7.0, "pac": 100.0, "power": f64::from(n), "total_power": 9.0
        }}}))
        .bind(start + chrono::Duration::hours(n.into()))
        .execute(&db)
        .await
        .unwrap();
    }

    sqlx::query("SELECT compress_chunk(c) FROM show_chunks('solar_data_tsdb') c")
        .execute(&db)
        .await
        .unwrap();

    let mut tx = db.begin().await.unwrap();
    home_gateway::adhoc::tasks::backfill_solar_kpis::backfill(&mut tx, 2)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let missing: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM solar_data_tsdb \
         WHERE today_kwh IS NULL OR month_kwh IS NULL OR total_kwh IS NULL",
    )
    .fetch_one(&db)
    .await
    .unwrap();
    let today_total: f64 = sqlx::query_scalar("SELECT sum(today_kwh) FROM solar_data_tsdb")
        .fetch_one(&db)
        .await
        .unwrap();

    assert_eq!(missing, 0);
    assert_eq!(today_total, 10.0);
}

#[tokio::test]
async fn current_reads_the_latest_kpis_and_yesterdays_last_total() {
    let db = fresh_database().await.pool;

    let now = Utc::now();
    let today = now
        .with_timezone(&chrono_tz::Australia::Perth)
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(chrono_tz::Australia::Perth)
        .unwrap()
        .with_timezone(&Utc);

    insert(&db, today - chrono::Duration::hours(3), 900.0, None).await;
    insert(&db, today - chrono::Duration::hours(1), 1200.0, None).await;
    insert(&db, now - chrono::Duration::minutes(1), 400.0, Some(3.0)).await;

    let solar = current(&db).await.unwrap();

    assert_eq!(solar.current_production_wh, 400.0);
    assert_eq!(solar.today_production_kwh, 4.0);
    assert_eq!(solar.yesterday_production_kwh, 12.0);
    assert_eq!(solar.month_production_kwh, 1.0);
    assert_eq!(solar.all_time_production_kwh, 3.0);
    assert_eq!(solar.uv_level, Some(3.0));
}
