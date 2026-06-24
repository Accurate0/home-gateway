use axum::Json;
use http::StatusCode;
use serde::Serialize;

use crate::actors::{
    alarm::AlarmActor,
    cron::CronActor,
    devices::{control_switch, plant_sensor, presence_sensor},
    door_sensor,
    eink_display::EInkDisplayActor,
    environment_sensor,
    events::{
        appliances::ApplianceEventsSupervisor, dispatcher::EventDispatcher,
        door_events::DoorEventsSupervisor,
    },
    light, push, smart_switch,
    solar::SolarIngestActor,
    synergy::SynergyActor,
    unifi::UnifiConnectedClientHandler,
    woolworths::WoolworthsActor,
    workflows::WorkflowWorker,
};

pub async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

/// Long-lived singleton actors that should always be registered while the
/// gateway is running. Each is spawned (and restarted on failure) by the root
/// supervisor or a spawn helper, so a missing entry here means that actor has
/// died and failed to come back.
const EXPECTED_ACTORS: &[&str] = &[
    EventDispatcher::NAME,
    UnifiConnectedClientHandler::NAME,
    CronActor::NAME,
    SynergyActor::NAME,
    WoolworthsActor::NAME,
    AlarmActor::NAME,
    EInkDisplayActor::NAME,
    SolarIngestActor::NAME,
    DoorEventsSupervisor::NAME,
    ApplianceEventsSupervisor::NAME,
    WorkflowWorker::NAME,
    push::PushWorker::NAME,
    light::LightHandler::NAME,
    control_switch::ControlSwitchHandler::NAME,
    smart_switch::SmartSwitchHandler::NAME,
    door_sensor::DoorSensorHandler::NAME,
    environment_sensor::EnvironmentSensorHandler::NAME,
    presence_sensor::PresenceSensorHandler::NAME,
    plant_sensor::PlantSensorHandler::NAME,
];

#[derive(Serialize)]
pub struct ActorStatus {
    name: &'static str,
    present: bool,
}

#[derive(Serialize)]
pub struct ActorHealth {
    healthy: bool,
    actors: Vec<ActorStatus>,
    registered: Vec<String>,
}

/// Reports liveness of the expected singleton actors. Returns `503` if any are
/// missing from the registry so external probes (k8s, uptime checks) can act on
/// a crashed actor that the supervisor couldn't restart.
pub async fn actor_health() -> (StatusCode, Json<ActorHealth>) {
    let actors: Vec<ActorStatus> = EXPECTED_ACTORS
        .iter()
        .map(|name| ActorStatus {
            name,
            present: ractor::registry::where_is(name.to_string()).is_some(),
        })
        .collect();

    let healthy = actors.iter().all(|a| a.present);
    let status = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(ActorHealth {
            healthy,
            actors,
            registered: ractor::registry::registered(),
        }),
    )
}
