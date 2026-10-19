use home_gateway::adhoc::cron_tasks::trim_workflow_runs::TrimWorkflowRuns;
use home_gateway::adhoc::{AdhocTaskContext, AnyAdhocCronTask};
use pretty_assertions::assert_eq;
use serial_test::serial;
use sqlx::{Pool, Postgres};

use crate::common::Harness;

/// Runs a cron task through its public entry point, in a committed transaction,
/// exactly as the adhoc runner does.
async fn run(harness: &Harness, task: &dyn AnyAdhocCronTask) -> u64 {
    let mut tx = harness.db.begin().await.unwrap();
    let affected = {
        let mut ctx = AdhocTaskContext::new(&harness.state, &mut tx);
        task.run(&mut ctx).await.unwrap()
    };
    tx.commit().await.unwrap();

    affected
}

async fn insert_workflow_run(db: &Pool<Postgres>, slug: &str, age_days: i32) {
    sqlx::query(
        "INSERT INTO workflow_runs (slug, name, event_id, outcome, dry_run, duration_ms, started_at) \
         VALUES ($1, $1, gen_random_uuid(), 'success', false, 1, now() - make_interval(days => $2))",
    )
    .bind(slug)
    .bind(age_days)
    .execute(db)
    .await
    .unwrap();
}

#[tokio::test]
#[serial]
async fn trim_workflow_runs_drops_past_the_cutoff_and_keeps_recent_ones() {
    let harness = Harness::start().await;

    insert_workflow_run(&harness.db, "ancient", 400).await;
    insert_workflow_run(&harness.db, "old", 91).await;
    insert_workflow_run(&harness.db, "fresh", 89).await;
    insert_workflow_run(&harness.db, "today", 0).await;

    let deleted = run(&harness, &TrimWorkflowRuns).await;

    assert_eq!(deleted, 2);

    let remaining: Vec<String> =
        sqlx::query_scalar("SELECT slug FROM workflow_runs ORDER BY started_at")
            .fetch_all(&harness.db)
            .await
            .unwrap();

    assert_eq!(remaining, vec!["fresh", "today"]);
}

#[tokio::test]
#[serial]
async fn hypertables_have_retention_policies() {
    let harness = Harness::start().await;

    let policies: Vec<(String, String)> = sqlx::query_as(
        "SELECT hypertable_name::text, config->>'drop_after' FROM timescaledb_information.jobs \
         WHERE proc_name = 'policy_retention' ORDER BY hypertable_name",
    )
    .fetch_all(&harness.db)
    .await
    .unwrap();

    let expected = [
        ("air_purifier_events", "180 days"),
        ("derived_door_events", "365 days"),
        ("device_metric", "180 days"),
        ("door_sensor", "365 days"),
        ("garage_door_events", "365 days"),
        ("home_assistant_events", "90 days"),
        ("jellyfin_playback_events", "180 days"),
        ("light_history", "90 days"),
        ("robot_vacuum_events", "180 days"),
        ("smart_switch", "180 days"),
        ("temperature_sensor", "180 days"),
        ("unraid_array_events", "365 days"),
        ("unraid_disk_events", "365 days"),
    ]
    .map(|(table, drop_after)| (table.to_owned(), drop_after.to_owned()));

    assert_eq!(policies, expected);
}
