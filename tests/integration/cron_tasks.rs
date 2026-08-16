use home_gateway::adhoc::cron_tasks::{
    trim_derived_door_events::TrimDerivedDoorEvents, trim_workflow_runs::TrimWorkflowRuns,
};
use home_gateway::adhoc::{AdhocCronTask, AdhocTaskContext};
use pretty_assertions::assert_eq;
use serial_test::serial;
use sqlx::{Pool, Postgres};

use crate::common::Harness;

/// Runs a cron task through its public entry point, in a committed transaction,
/// exactly as the adhoc runner does.
async fn run(harness: &Harness, task: &dyn AdhocCronTask) -> u64 {
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

async fn insert_door_event(db: &Pool<Postgres>, id: &str, age_days: i32) {
    sqlx::query(
        "INSERT INTO derived_door_events (event_id, name, id, ieee_addr, state, \"time\") \
         VALUES (gen_random_uuid(), $1, $1, '0x00', 'open', now() - make_interval(days => $2))",
    )
    .bind(id)
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
async fn trim_derived_door_events_drops_chunks_past_the_cutoff() {
    let harness = Harness::start().await;

    insert_door_event(&harness.db, "ancient", 800).await;
    insert_door_event(&harness.db, "recent", 1).await;

    let chunks = run(&harness, &TrimDerivedDoorEvents).await;

    assert_eq!(chunks, 1);

    let remaining: Vec<String> = sqlx::query_scalar("SELECT id FROM derived_door_events")
        .fetch_all(&harness.db)
        .await
        .unwrap();

    assert_eq!(remaining, vec!["recent"]);
}
