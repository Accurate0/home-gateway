use home_gateway::repo::environment::{EnvironmentReading, EnvironmentRepo};
use home_gateway::repo::robot_vacuum::RobotVacuumRepo;
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::common::db::fresh_database;

fn reading(temperature: f64, humidity: Option<f64>) -> EnvironmentReading {
    EnvironmentReading {
        event_id: Uuid::new_v4(),
        id: Some("study".to_owned()),
        friendly_name: "study sensor".to_owned(),
        ieee_addr: "0x00158d0001".to_owned(),
        temperature,
        battery: Some(88),
        humidity,
        pressure: None,
        pm25: None,
        voc_index: None,
        lux: None,
        uv_index: None,
    }
}

#[tokio::test]
async fn an_environment_reading_lands_in_both_the_history_and_the_latest_row() {
    let pool = fresh_database().await.pool;
    let repo = EnvironmentRepo::new(pool.clone());

    repo.record(&reading(21.5, Some(40.0))).await.unwrap();

    let history_count =
        sqlx::query_scalar!(r#"SELECT count(*) as "count!" FROM temperature_sensor"#)
            .fetch_one(&pool)
            .await
            .unwrap();

    let latest = repo.latest("study").await.unwrap().unwrap();

    assert_eq!(history_count, 1);
    assert_eq!(latest.temperature, 21.5);
    assert_eq!(latest.humidity, Some(40.0));
}

#[tokio::test]
async fn a_later_environment_reading_replaces_the_latest_row_but_appends_history() {
    let pool = fresh_database().await.pool;
    let repo = EnvironmentRepo::new(pool.clone());

    repo.record(&reading(21.5, Some(40.0))).await.unwrap();
    repo.record(&reading(23.0, None)).await.unwrap();

    let history_count =
        sqlx::query_scalar!(r#"SELECT count(*) as "count!" FROM temperature_sensor"#)
            .fetch_one(&pool)
            .await
            .unwrap();

    let latest = repo.latest("study").await.unwrap().unwrap();

    assert_eq!(history_count, 2);
    assert_eq!(latest.temperature, 23.0);
    assert_eq!(latest.humidity, None);
}

#[tokio::test]
async fn a_partial_robot_vacuum_update_keeps_the_fields_it_does_not_carry() {
    let pool = fresh_database().await.pool;
    let repo = RobotVacuumRepo::new(pool.clone());

    repo.record_valetudo_state(
        Uuid::new_v4(),
        "vacuum",
        Some("cleaning"),
        Some(80),
        Some("high"),
    )
    .await
    .unwrap();

    repo.record_valetudo_state(Uuid::new_v4(), "vacuum", None, Some(75), None)
        .await
        .unwrap();

    let latest = repo.latest_many(&["vacuum".to_owned()]).await.unwrap();
    let row = latest.first().unwrap();

    assert_eq!(row.state.as_deref(), Some("cleaning"));
    assert_eq!(row.battery_level, Some(75));
    assert_eq!(row.fan_speed.as_deref(), Some("high"));
}

#[tokio::test]
async fn robot_vacuum_events_are_appended_per_report() {
    let pool = fresh_database().await.pool;
    let repo = RobotVacuumRepo::new(pool.clone());

    repo.record_valetudo_state(Uuid::new_v4(), "vacuum", Some("cleaning"), Some(80), None)
        .await
        .unwrap();

    repo.record_roborock_status(Uuid::new_v4(), "vacuum", "docked")
        .await
        .unwrap();

    let events = sqlx::query_scalar!(r#"SELECT count(*) as "count!" FROM robot_vacuum_events"#)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(events, 2);
}
