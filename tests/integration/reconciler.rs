use std::time::Duration;

use home_gateway::actors::devices::handler::spawn_handler;
use home_gateway::actors::devices::light::{Entity, LightHandler, LightHandlerMessage, NewEvent};
use home_gateway::actors::devices::smart_switch::{self, SmartSwitchHandler};
use home_gateway::actors::system::reconciler::{ReconcilerSweeper, spawn_reconciler};
use home_gateway::actors::system::rpc;
use home_gateway::event_bus::EventBusMessage;
use home_gateway::repo::intent::{DeviceKind, IntentAttributes, PowerState, SwitchAttributes};
use home_gateway::repo::light::{LightAttributes, LightState};
use pretty_assertions::assert_eq;
use ractor::Actor;
use serial_test::serial;
use uuid::Uuid;

use crate::common::{Harness, wait_for};

const DB_TIMEOUT: Duration = Duration::from_secs(10);
const LAMP: &str = "0x0000000000000003";
const LAMP_TOPIC: &str = "zigbee2mqtt/0x0000000000000003/set";

async fn start() -> Harness {
    let harness = Harness::start().await;

    spawn_handler::<LightHandler>(&harness.root, harness.state.clone())
        .await
        .unwrap();
    spawn_handler::<SmartSwitchHandler>(&harness.root, harness.state.clone())
        .await
        .unwrap();

    harness
}

async fn start_with_reconciler() -> Harness {
    let harness = start().await;

    spawn_reconciler(&harness.root, harness.state.clone())
        .await
        .unwrap();

    Actor::spawn(
        Some(ReconcilerSweeper::NAME.to_owned()),
        ReconcilerSweeper {
            shared_actor_state: harness.state.clone(),
        },
        (),
    )
    .await
    .expect("failed to spawn the reconciler sweeper");

    harness
}

fn turn_on() {
    rpc::cast_factory(
        LightHandler::NAME,
        LightHandlerMessage::TurnOn {
            ieee_addr: LAMP.to_owned(),
        },
    )
    .unwrap();
}

fn report(attributes: LightAttributes) {
    rpc::cast_factory(
        LightHandler::NAME,
        LightHandlerMessage::NewEvent(Box::new(NewEvent {
            event_id: Uuid::new_v4(),
            traceparent: None,
            entity: Entity::Zigbee {
                address: LAMP.to_owned(),
                attributes,
            },
        })),
    )
    .unwrap();
}

async fn intent_status(harness: &Harness) -> Option<String> {
    sqlx::query_scalar(
        "SELECT status FROM device_intent WHERE address = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(LAMP)
    .fetch_optional(&harness.db)
    .await
    .unwrap()
}

async fn wait_for_status(harness: &Harness, want: &str) {
    wait_for(DB_TIMEOUT, &format!("intent status {want}"), || async {
        intent_status(harness).await.filter(|status| status == want)
    })
    .await;
}

async fn history_rows(harness: &Harness) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM light_history WHERE address = $1")
        .bind(LAMP)
        .fetch_one(&harness.db)
        .await
        .unwrap()
}

#[tokio::test]
#[serial]
async fn a_command_records_a_pending_intent_and_no_history() {
    let harness = start().await;

    turn_on();

    let payload = harness.recorder.expect_publish(LAMP_TOPIC).await;
    assert_eq!(payload, serde_json::json!({ "state": "ON" }));

    wait_for_status(&harness, "pending").await;

    assert_eq!(
        history_rows(&harness).await,
        0,
        "an unconfirmed command must not write light history"
    );
}

#[tokio::test]
#[serial]
async fn a_device_report_confirms_the_intent_and_writes_one_history_row() {
    let harness = start().await;

    turn_on();
    harness.recorder.expect_publish(LAMP_TOPIC).await;
    wait_for_status(&harness, "pending").await;

    report(LightAttributes::state("ON"));

    wait_for_status(&harness, "confirmed").await;

    assert_eq!(
        history_rows(&harness).await,
        1,
        "the device report is the only thing that writes history"
    );
}

#[tokio::test]
#[serial]
async fn a_smart_switch_report_confirms_the_intent_of_a_switch_acting_as_a_light() {
    let harness = start().await;

    turn_on();
    harness.recorder.expect_publish(LAMP_TOPIC).await;
    wait_for_status(&harness, "pending").await;

    rpc::cast_factory(
        SmartSwitchHandler::NAME,
        smart_switch::Message::NewEvent(smart_switch::NewEvent {
            event_id: Uuid::new_v4(),
            traceparent: None,
            entity: smart_switch::Entity::Zigbee {
                address: LAMP.to_owned(),
                friendly_name: "Test Lamp".to_owned(),
                voltage: 240,
                power: 15,
                current: 0.06,
                energy: 1.5,
                state: Some("ON".to_owned()),
            },
        }),
    )
    .unwrap();

    wait_for_status(&harness, "confirmed").await;

    assert_eq!(
        history_rows(&harness).await,
        1,
        "the forwarded switch report is what writes light history"
    );
}

#[tokio::test]
#[serial]
async fn a_partial_report_confirms_a_brightness_only_command() {
    let harness = start().await;

    rpc::cast_factory(
        LightHandler::NAME,
        LightHandlerMessage::SetBrightness {
            ieee_addr: LAMP.to_owned(),
            value: 120,
        },
    )
    .unwrap();

    harness.recorder.expect_publish(LAMP_TOPIC).await;
    wait_for_status(&harness, "pending").await;

    report(LightAttributes {
        state: Some("ON".to_owned()),
        brightness: Some(120),
        colour: Some("#ffffff".to_owned()),
        ..LightAttributes::default()
    });

    wait_for_status(&harness, "confirmed").await;
}

#[tokio::test]
#[serial]
async fn a_mismatched_report_leaves_the_intent_pending() {
    let harness = start().await;

    rpc::cast_factory(
        LightHandler::NAME,
        LightHandlerMessage::SetBrightness {
            ieee_addr: LAMP.to_owned(),
            value: 120,
        },
    )
    .unwrap();

    harness.recorder.expect_publish(LAMP_TOPIC).await;
    wait_for_status(&harness, "pending").await;

    report(LightAttributes {
        state: Some("ON".to_owned()),
        brightness: Some(10),
        ..LightAttributes::default()
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    assert_eq!(
        intent_status(&harness).await.as_deref(),
        Some("pending"),
        "a report that does not satisfy the command must not confirm it"
    );
}

#[tokio::test]
#[serial]
async fn a_new_command_supersedes_the_pending_one() {
    let harness = start().await;

    turn_on();
    wait_for_status(&harness, "pending").await;

    rpc::cast_factory(
        LightHandler::NAME,
        LightHandlerMessage::TurnOff {
            ieee_addr: LAMP.to_owned(),
        },
    )
    .unwrap();

    wait_for(DB_TIMEOUT, "the first intent to be superseded", || async {
        let superseded: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM device_intent WHERE address = $1 AND status = 'superseded'",
        )
        .bind(LAMP)
        .fetch_one(&harness.db)
        .await
        .unwrap();

        (superseded == 1).then_some(superseded)
    })
    .await;

    let pending: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM device_intent WHERE address = $1 AND status = 'pending'",
    )
    .bind(LAMP)
    .fetch_one(&harness.db)
    .await
    .unwrap();

    assert_eq!(pending, 1, "only the newest command stays pending");
}

#[tokio::test]
#[serial]
async fn an_unconfirmed_command_is_re_driven_then_fails() {
    let harness = start_with_reconciler().await;
    let mut events = harness.subscribe();

    turn_on();

    let failed = events
        .next_matching("a command failure", |message| {
            matches!(message, EventBusMessage::CommandFailed { address, .. } if address == LAMP)
        })
        .await;

    let EventBusMessage::CommandFailed {
        attempts, address, ..
    } = failed
    else {
        panic!("expected a CommandFailed event");
    };

    assert_eq!(address, LAMP);
    assert_eq!(attempts, 3, "two re-drives, then the attempt that gave up");

    wait_for_status(&harness, "failed").await;

    let publishes = harness
        .recorder
        .published()
        .into_iter()
        .filter(|(topic, _)| topic == LAMP_TOPIC)
        .count();

    assert!(
        publishes >= 2,
        "the command should have been re-driven, saw {publishes} publish(es)"
    );
}

#[tokio::test]
#[serial]
async fn a_confirmed_command_is_never_re_driven() {
    let harness = start_with_reconciler().await;

    turn_on();
    harness.recorder.expect_publish(LAMP_TOPIC).await;

    report(LightAttributes::state("ON"));
    wait_for_status(&harness, "confirmed").await;

    let after_confirm = harness
        .recorder
        .published()
        .into_iter()
        .filter(|(topic, _)| topic == LAMP_TOPIC)
        .count();

    tokio::time::sleep(Duration::from_millis(600)).await;

    let now = harness
        .recorder
        .published()
        .into_iter()
        .filter(|(topic, _)| topic == LAMP_TOPIC)
        .count();

    assert_eq!(
        now, after_confirm,
        "a confirmed command must not be re-driven"
    );
}

#[tokio::test]
#[serial]
async fn a_relative_move_is_never_re_driven() {
    let harness = start_with_reconciler().await;

    rpc::cast_factory(
        LightHandler::NAME,
        LightHandlerMessage::BrightnessMove {
            ieee_addr: LAMP.to_owned(),
            value: 40,
            on_off: false,
        },
    )
    .unwrap();

    harness.recorder.expect_publish(LAMP_TOPIC).await;
    wait_for_status(&harness, "pending").await;

    tokio::time::sleep(Duration::from_millis(600)).await;

    let publishes = harness
        .recorder
        .published()
        .into_iter()
        .filter(|(topic, _)| topic == LAMP_TOPIC)
        .count();

    assert_eq!(publishes, 1, "a relative move must never be re-sent");
    assert_eq!(intent_status(&harness).await.as_deref(), Some("pending"));

    report(LightAttributes::state("ON"));

    wait_for_status(&harness, "confirmed").await;
}

#[tokio::test]
#[serial]
async fn concurrent_sweeps_never_claim_the_same_intent() {
    let harness = start().await;
    let repo = harness.state.repos.intent();

    for index in 0..25 {
        repo.replace_pending(
            &format!("0xaaaa{index:012}"),
            &IntentAttributes::Light(LightAttributes::state("ON")),
            false,
            Uuid::new_v4(),
        )
        .await
        .unwrap();
    }

    let grace = chrono::TimeDelta::zero();
    let backoff = chrono::TimeDelta::hours(1);

    let (left, right) = tokio::join!(
        repo.claim_due(grace, backoff, 64),
        repo.claim_due(grace, backoff, 64),
    );

    let mut claimed: Vec<i64> = left
        .unwrap()
        .into_iter()
        .chain(right.unwrap())
        .map(|intent| intent.id)
        .collect();

    let total = claimed.len();
    claimed.sort_unstable();
    claimed.dedup();

    assert_eq!(
        claimed.len(),
        total,
        "an intent was claimed by both sweeps, so it would be re-driven twice"
    );
}

#[tokio::test]
#[serial]
async fn each_device_kind_round_trips_through_the_intent_table() {
    let harness = start().await;
    let repo = harness.state.repos.intent();

    let light = IntentAttributes::Light(LightAttributes::state("ON"));
    let switch = IntentAttributes::SmartSwitch(SwitchAttributes {
        state: PowerState::Off,
    });

    repo.replace_pending("0xbbbb00000001", &light, false, Uuid::new_v4())
        .await
        .unwrap();
    repo.replace_pending("0xbbbb00000002", &switch, false, Uuid::new_v4())
        .await
        .unwrap();

    let stored_light = repo
        .pending_for(DeviceKind::Light, "0xbbbb00000001")
        .await
        .unwrap();
    let stored_switch = repo
        .pending_for(DeviceKind::SmartSwitch, "0xbbbb00000002")
        .await
        .unwrap();

    assert_eq!(stored_light.len(), 1);
    assert_eq!(stored_light[0].attributes, light);
    assert_eq!(stored_light[0].kind(), DeviceKind::Light);

    assert_eq!(stored_switch.len(), 1);
    assert_eq!(stored_switch[0].attributes, switch);
    assert_eq!(stored_switch[0].kind(), DeviceKind::SmartSwitch);

    let cross_kind = repo
        .pending_for(DeviceKind::Light, "0xbbbb00000002")
        .await
        .unwrap();

    assert!(
        cross_kind.is_empty(),
        "a switch intent must not surface under the light kind"
    );
}

#[tokio::test]
#[serial]
async fn a_subset_match_ignores_unrequested_attributes() {
    let wanted = LightAttributes {
        brightness: Some(120),
        ..LightAttributes::default()
    };

    assert!(wanted.satisfied_by(&LightState {
        on: true,
        brightness: Some(120),
        colour_temp: Some(370),
        colour: Some("#ffffff".to_owned()),
    }));

    assert!(!wanted.satisfied_by(&LightState {
        on: true,
        brightness: Some(10),
        colour_temp: None,
        colour: None,
    }));
}
