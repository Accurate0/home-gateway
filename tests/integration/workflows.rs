use std::time::Duration;

use home_gateway::actors::devices::light::spawn::spawn_light_handler;
use home_gateway::actors::workflows::{dispatcher::WorkflowDispatcher, spawn::spawn_workflows};
use home_gateway::event_bus::EventBusMessage;
use pretty_assertions::assert_eq;
use ractor::Actor;
use serial_test::serial;
use uuid::Uuid;

use crate::common::{Harness, wait_for};

const DB_TIMEOUT: Duration = Duration::from_secs(10);
const LAMP_TOPIC: &str = "zigbee2mqtt/0x0000000000000003/set";
const DOOR_ADDRESS: &str = "0x0000000000000001";
const MOTION_ADDRESS: &str = "0x0000000000000004";

async fn start() -> Harness {
    let harness = Harness::start().await;

    spawn_workflows(&harness.root, harness.state.clone())
        .await
        .unwrap();
    spawn_light_handler(&harness.root, harness.state.clone())
        .await
        .unwrap();

    Actor::spawn(
        Some(WorkflowDispatcher::NAME.to_owned()),
        WorkflowDispatcher {
            shared_actor_state: harness.state.clone(),
        },
        (),
    )
    .await
    .expect("failed to spawn the workflow dispatcher");

    harness
}

async fn assert_ran(harness: &Harness, slug: &str) {
    wait_for(
        DB_TIMEOUT,
        &format!("a workflow run for {slug}"),
        || async {
            let count: i64 =
                sqlx::query_scalar("SELECT count(*) FROM workflow_runs WHERE slug = $1")
                    .bind(slug)
                    .fetch_one(&harness.db)
                    .await
                    .unwrap();

            (count > 0).then_some(count)
        },
    )
    .await;
}

#[tokio::test]
#[serial]
async fn a_door_event_runs_the_workflow_and_publishes_to_mqtt() {
    let harness = start().await;

    harness.event_bus.publish(EventBusMessage::Door {
        event_id: Uuid::new_v4(),
        ieee_addr: DOOR_ADDRESS.to_owned(),
        open: true,
    });

    // The trigger runs `test-door-opens-lamp-on`, which nests into
    // `test-lamp-on`, which turns the lamp on over MQTT. Only the dispatched
    // workflow gets a `workflow_runs` row; the nested one runs inline under it,
    // so the publish is what proves the nesting happened.
    let payload = harness.recorder.expect_publish(LAMP_TOPIC).await;
    assert_eq!(payload, serde_json::json!({ "state": "ON" }));

    assert_ran(&harness, "test-door-opens-lamp-on").await;
}

#[tokio::test]
#[serial]
async fn a_presence_clear_event_turns_the_lamp_off() {
    let harness = start().await;

    harness.event_bus.publish(EventBusMessage::Presence {
        event_id: Uuid::new_v4(),
        sensor: MOTION_ADDRESS.to_owned(),
        present: false,
    });

    let payload = harness.recorder.expect_publish(LAMP_TOPIC).await;
    assert_eq!(payload, serde_json::json!({ "state": "OFF" }));

    assert_ran(&harness, "test-motion-clear-lamp-off").await;
}

#[tokio::test]
#[serial]
async fn a_non_matching_trigger_runs_nothing() {
    let harness = start().await;

    // No fixture workflow triggers on presence becoming true.
    harness.event_bus.publish(EventBusMessage::Presence {
        event_id: Uuid::new_v4(),
        sensor: MOTION_ADDRESS.to_owned(),
        present: true,
    });

    tokio::time::sleep(Duration::from_millis(750)).await;

    harness.recorder.assert_no_publish(LAMP_TOPIC);

    let runs: i64 = sqlx::query_scalar("SELECT count(*) FROM workflow_runs")
        .fetch_one(&harness.db)
        .await
        .unwrap();

    assert_eq!(runs, 0, "no workflow should have run");
}
