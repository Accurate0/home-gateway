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
    events::door_events::DoorEventsSupervisor,
    light, push, smart_switch,
    solar::SolarIngestActor,
    synergy::SynergyActor,
    unifi::UnifiConnectedClientHandler,
    woolworths::WoolworthsActor,
    workflows::{WorkflowWorker, dispatcher::WorkflowDispatcher},
};

pub async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

const EXPECTED_ACTORS: &[&str] = &[
    WorkflowDispatcher::NAME,
    UnifiConnectedClientHandler::NAME,
    CronActor::NAME,
    SynergyActor::NAME,
    WoolworthsActor::NAME,
    AlarmActor::NAME,
    EInkDisplayActor::NAME,
    SolarIngestActor::NAME,
    DoorEventsSupervisor::NAME,
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
