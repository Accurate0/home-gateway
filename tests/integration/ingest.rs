use std::time::Duration;

use home_gateway::actors::devices::{
    door_events::DoorEventsSupervisor, door_sensor::spawn::spawn_door_handler,
    environment_sensor::spawn::spawn_environment_sensor_handler,
};
use home_gateway::actors::system::mqtt_ingest::{self, MqttIngest, spawn::spawn_mqtt_ingest};
use home_gateway::event_bus::{EventBusMessage, SensorReading};
use pretty_assertions::assert_eq;
use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, JobOptions},
};
use serial_test::serial;

use crate::common::{Harness, wait_for};

const DB_TIMEOUT: Duration = Duration::from_secs(10);

const DOOR_ADDRESS: &str = "0x0000000000000001";
const ENVIRONMENT_ADDRESS: &str = "0x0000000000000002";

/// `EventBusMessage::Door` is published by `DerivedDoorEvents` (the debounced,
/// confirmed transition), not by the door sensor handler itself.
async fn spawn_door_events(harness: &Harness) {
    ractor::Actor::spawn_linked(
        Some(DoorEventsSupervisor::NAME.to_owned()),
        DoorEventsSupervisor {
            shared_actor_state: harness.state.clone(),
        },
        (),
        harness.root.get_cell(),
    )
    .await
    .expect("failed to spawn the door events supervisor");
}

fn ingest_actor() -> ActorRef<FactoryMessage<(), mqtt_ingest::Message>> {
    ractor::registry::where_is(MqttIngest::NAME)
        .map(ActorRef::from)
        .expect("the mqtt ingest actor should be registered")
}

fn publish(topic: &str, payload: serde_json::Value) {
    ingest_actor()
        .send_message(FactoryMessage::Dispatch(Job {
            key: (),
            msg: mqtt_ingest::Message::MqttPacket {
                payload: serde_json::to_vec(&payload).unwrap().into(),
                topic: topic.to_owned(),
            },
            options: JobOptions::default(),
            accepted: None,
        }))
        .expect("failed to dispatch the mqtt packet");
}

/// zigbee payloads are matched to a device by the `device.ieee_addr` field, not
/// by the topic, so every fixture payload carries one.
fn zigbee_payload(address: &str, mut fields: serde_json::Value) -> serde_json::Value {
    fields["device"] = serde_json::json!({ "ieee_addr": address });
    fields
}

#[tokio::test]
#[serial]
async fn zigbee_environment_report_lands_in_the_db_and_on_the_bus() {
    let harness = Harness::start().await;
    spawn_mqtt_ingest(&harness.root, harness.state.clone())
        .await
        .unwrap();
    spawn_environment_sensor_handler(&harness.root, harness.state.clone())
        .await
        .unwrap();

    let mut events = harness.subscribe();

    publish(
        "zigbee2mqtt/test-environment",
        zigbee_payload(
            ENVIRONMENT_ADDRESS,
            serde_json::json!({
                "temperature": 21.5,
                "humidity": 48.0,
                "pressure": 1013.0,
                "battery": 87,
            }),
        ),
    );

    let event = events
        .next_matching("an environment event", |message| {
            matches!(message, EventBusMessage::Environment { .. })
        })
        .await;

    let EventBusMessage::Environment {
        sensor, readings, ..
    } = event
    else {
        unreachable!("the predicate only matches environment events")
    };

    assert_eq!(sensor, ENVIRONMENT_ADDRESS);
    assert!(
        readings.contains(&SensorReading::Temperature { value: 21.5 }),
        "expected the temperature reading on the bus, got {readings:?}"
    );

    let latest = wait_for(
        DB_TIMEOUT,
        "the latest_temperature_sensor upsert",
        || async {
            sqlx::query_scalar::<_, Option<f64>>(
                "SELECT temperature FROM latest_temperature_sensor WHERE entity_id = $1",
            )
            .bind("test-room")
            .fetch_optional(&harness.db)
            .await
            .unwrap()
        },
    )
    .await;

    assert_eq!(latest, Some(21.5));

    let history: i64 =
        sqlx::query_scalar("SELECT count(*) FROM temperature_sensor WHERE ieee_addr = $1")
            .bind(ENVIRONMENT_ADDRESS)
            .fetch_one(&harness.db)
            .await
            .unwrap();

    assert_eq!(history, 1, "expected one historical reading");
}

#[tokio::test]
#[serial]
async fn zigbee_door_report_lands_in_the_db_and_on_the_bus() {
    let harness = Harness::start().await;
    spawn_mqtt_ingest(&harness.root, harness.state.clone())
        .await
        .unwrap();
    spawn_door_handler(&harness.root, harness.state.clone())
        .await
        .unwrap();
    spawn_door_events(&harness).await;

    let mut events = harness.subscribe();

    publish(
        "zigbee2mqtt/test-door",
        zigbee_payload(
            DOOR_ADDRESS,
            serde_json::json!({ "contact": false, "battery": 91 }),
        ),
    );

    let event = events
        .next_matching("a door event", |message| {
            matches!(message, EventBusMessage::Door { .. })
        })
        .await;

    assert_eq!(
        event.entity(),
        DOOR_ADDRESS,
        "the door event should carry the reporting device"
    );

    let count = wait_for(DB_TIMEOUT, "the door_sensor row", || async {
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM door_sensor WHERE ieee_addr = $1")
                .bind(DOOR_ADDRESS)
                .fetch_one(&harness.db)
                .await
                .unwrap();

        (count > 0).then_some(count)
    })
    .await;

    assert_eq!(count, 1);
}

#[tokio::test]
#[serial]
async fn free_form_zigbee_metrics_land_in_the_generic_sink() {
    let harness = Harness::start().await;
    spawn_mqtt_ingest(&harness.root, harness.state.clone())
        .await
        .unwrap();
    spawn_door_handler(&harness.root, harness.state.clone())
        .await
        .unwrap();
    spawn_door_events(&harness).await;

    publish(
        "zigbee2mqtt/test-door",
        zigbee_payload(
            DOOR_ADDRESS,
            serde_json::json!({
                "contact": true,
                "battery": 91,
                "device_temperature": 23.5,
                "voltage": 3021,
            }),
        ),
    );

    // `device_temperature` is declared float and `voltage` int in the fixture
    // model profile, so both route to the numeric column.
    let metrics = wait_for(DB_TIMEOUT, "the device_metric rows", || async {
        let rows: Vec<(String, Option<f64>, Option<String>)> = sqlx::query_as(
            "SELECT metric, value, text_value FROM device_metric WHERE address = $1 ORDER BY metric",
        )
        .bind(DOOR_ADDRESS)
        .fetch_all(&harness.db)
        .await
        .unwrap();

        (rows.len() >= 2).then_some(rows)
    })
    .await;

    let device_temperature = metrics
        .iter()
        .find(|(metric, ..)| metric == "device_temperature")
        .expect("expected a device_temperature metric");
    assert_eq!(device_temperature.1, Some(23.5));
    assert_eq!(device_temperature.2, None);

    let voltage = metrics
        .iter()
        .find(|(metric, ..)| metric == "voltage")
        .expect("expected a voltage metric");
    assert_eq!(voltage.1, Some(3021.0));
    assert_eq!(voltage.2, None);
}

#[tokio::test]
#[serial]
async fn an_unregistered_address_is_ignored() {
    let harness = Harness::start().await;
    spawn_mqtt_ingest(&harness.root, harness.state.clone())
        .await
        .unwrap();
    spawn_environment_sensor_handler(&harness.root, harness.state.clone())
        .await
        .unwrap();

    publish(
        "zigbee2mqtt/who-is-this",
        zigbee_payload(
            "0x00000000deadbeef",
            serde_json::json!({ "temperature": 30.0 }),
        ),
    );

    tokio::time::sleep(Duration::from_millis(500)).await;

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM temperature_sensor")
        .fetch_one(&harness.db)
        .await
        .unwrap();

    assert_eq!(count, 0, "an unregistered address should not be persisted");
    assert!(
        ingest_actor().get_status() == ractor::ActorStatus::Running,
        "the ingest actor should survive an unknown device"
    );
}
