use std::time::Duration;

use chrono::{TimeDelta, Utc};
use home_gateway::actors::devices::handler::spawn_handler;
use home_gateway::actors::devices::light::LightHandler;
use home_gateway::actors::devices::presence_sensor::{
    Entity, Message as PresenceMessage, NewEvent, PresenceSensorHandler,
};
use home_gateway::actors::workflows::{dispatcher::WorkflowDispatcher, spawn::spawn_workflows};
use home_gateway::event_bus::EventBusMessage;
use home_gateway::repo::{timer_kind::TimerKind, workflow::NewPendingTimer};
use pretty_assertions::assert_eq;
use ractor::{
    Actor, ActorRef,
    factory::{FactoryMessage, Job, JobOptions},
};
use serial_test::serial;
use uuid::Uuid;

use crate::common::{Harness, wait_for};

const DB_TIMEOUT: Duration = Duration::from_secs(10);
const LAMP_TOPIC: &str = "zigbee2mqtt/0x0000000000000003/set";
const DOOR_ADDRESS: &str = "0x0000000000000001";
const MOTION_ADDRESS: &str = "0x0000000000000004";
const HELD_SLUG: &str = "test-presence-held-lamp-on";

async fn start() -> Harness {
    let harness = start_without_dispatcher().await;

    spawn_dispatcher(&harness).await;

    harness
}

async fn start_without_dispatcher() -> Harness {
    let harness = Harness::start().await;

    spawn_workflows(&harness.root, harness.state.clone())
        .await
        .unwrap();
    spawn_handler::<LightHandler>(&harness.root, harness.state.clone())
        .await
        .unwrap();

    harness
}

async fn spawn_dispatcher(harness: &Harness) {
    Actor::spawn(
        Some(WorkflowDispatcher::NAME.to_owned()),
        WorkflowDispatcher {
            shared_actor_state: harness.state.clone(),
        },
        (),
    )
    .await
    .expect("failed to spawn the workflow dispatcher");
}

async fn spawn_presence(harness: &Harness) -> ActorRef<FactoryMessage<(), PresenceMessage>> {
    spawn_handler::<PresenceSensorHandler>(&harness.root, harness.state.clone())
        .await
        .expect("failed to spawn the presence handler")
}

fn report_presence(handler: &ActorRef<FactoryMessage<(), PresenceMessage>>, present: bool) {
    handler
        .send_message(FactoryMessage::Dispatch(Job {
            key: (),
            msg: PresenceMessage::NewEvent(NewEvent {
                event_id: Uuid::new_v4(),
                entity: Entity::Zigbee {
                    address: MOTION_ADDRESS.to_owned(),
                    presence: present,
                },
                traceparent: None,
            }),
            options: JobOptions::default(),
            accepted: None,
        }))
        .expect("failed to dispatch the presence event");
}

async fn runs_for(harness: &Harness, slug: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM workflow_runs WHERE slug = $1")
        .bind(slug)
        .fetch_one(&harness.db)
        .await
        .unwrap()
}

async fn pending_timers(harness: &Harness) -> usize {
    harness
        .state
        .repos
        .workflow()
        .pending_timers()
        .await
        .unwrap()
        .len()
}

async fn arm_overdue_delay(harness: &Harness, overdue: TimeDelta) {
    harness
        .state
        .repos
        .workflow()
        .arm_timer(NewPendingTimer {
            workflow: "Test lamp on when door opens",
            kind: TimerKind::Delay,
            subject_kind: "door",
            subject_entity: DOOR_ADDRESS,
            event_id: Uuid::new_v4(),
            vars: serde_json::json!({}),
            fire_at: Utc::now() - overdue,
        })
        .await
        .unwrap()
        .expect("the delay timer should be armed");
}

async fn assert_ran(harness: &Harness, slug: &str) {
    wait_for(
        DB_TIMEOUT,
        &format!("a workflow run for {slug}"),
        || async {
            let count = runs_for(harness, slug).await;

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

#[tokio::test]
#[serial]
async fn a_switch_step_drives_a_smart_switch_declared_as_a_light() {
    let harness = start().await;

    harness.event_bus.publish(EventBusMessage::Door {
        event_id: Uuid::new_v4(),
        ieee_addr: DOOR_ADDRESS.to_owned(),
        open: false,
    });

    let payload = harness.recorder.expect_publish(LAMP_TOPIC).await;
    assert_eq!(payload, serde_json::json!({ "state": "ON" }));

    assert_ran(&harness, "test-switch-step").await;
}

#[tokio::test]
#[serial]
async fn a_held_presence_fires_once_its_duration_passes() {
    let harness = start().await;
    let presence = spawn_presence(&harness).await;

    report_presence(&presence, true);

    let payload = harness.recorder.expect_publish(LAMP_TOPIC).await;
    assert_eq!(payload, serde_json::json!({ "state": "ON" }));

    assert_ran(&harness, HELD_SLUG).await;
    assert_eq!(pending_timers(&harness).await, 0);
}

#[tokio::test]
#[serial]
async fn a_contrary_event_cancels_a_hold() {
    let harness = start().await;
    let presence = spawn_presence(&harness).await;

    report_presence(&presence, true);

    wait_for(DB_TIMEOUT, "the hold timer to be armed", || async {
        (pending_timers(&harness).await > 0).then_some(())
    })
    .await;

    report_presence(&presence, false);

    assert_ran(&harness, "test-motion-clear-lamp-off").await;

    tokio::time::sleep(Duration::from_millis(1500)).await;

    assert_eq!(runs_for(&harness, HELD_SLUG).await, 0);
    assert_eq!(pending_timers(&harness).await, 0);
}

#[tokio::test]
#[serial]
async fn an_overdue_timer_within_catch_up_fires_on_start() {
    let harness = start_without_dispatcher().await;

    arm_overdue_delay(&harness, TimeDelta::minutes(1)).await;
    spawn_dispatcher(&harness).await;

    let payload = harness.recorder.expect_publish(LAMP_TOPIC).await;
    assert_eq!(payload, serde_json::json!({ "state": "ON" }));

    assert_ran(&harness, "test-door-opens-lamp-on").await;
    assert_eq!(pending_timers(&harness).await, 0);
}

#[tokio::test]
#[serial]
async fn an_overdue_timer_past_catch_up_is_dropped() {
    let harness = start_without_dispatcher().await;

    arm_overdue_delay(&harness, TimeDelta::hours(1)).await;
    spawn_dispatcher(&harness).await;

    tokio::time::sleep(Duration::from_millis(750)).await;

    harness.recorder.assert_no_publish(LAMP_TOPIC);
    assert_eq!(runs_for(&harness, "test-door-opens-lamp-on").await, 0);
    assert_eq!(pending_timers(&harness).await, 0);
}
