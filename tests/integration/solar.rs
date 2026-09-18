use chrono::{DateTime, TimeZone, Utc};
use home_gateway::integrations::solar::queries::{
    SolarQueryError, current, history_since, statistics,
};
use pretty_assertions::assert_eq;
use sqlx::{Pool, Postgres};

use crate::common::db::fresh_database;

async fn insert(db: &Pool<Postgres>, at: DateTime<Utc>, kwh: f64, uv: Option<f64>) {
    sqlx::query(
        "INSERT INTO solar_data_tsdb (current_kwh, raw_data, uv_level, temperature, time) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(kwh)
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
