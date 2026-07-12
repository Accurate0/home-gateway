//! Condition evaluation, shared between two callers:
//! - workflow step `when` guards ([`super::WorkflowWorker`]), and
//! - trigger `when` gates ([`crate::actors::workflows::dispatcher`]).
//!
//! Both need to answer the same boolean predicates against live device/sensor
//! state, so the actor-query RPC logic lives here once and takes a plain
//! [`SharedActorState`] rather than being tied to the workflow worker.

use super::WorkflowError;
use crate::actors::sun::calc;
use crate::{
    actors::{
        devices::environment_sensor::{
            EnvironmentSensorHandler, LatestReading, Message as EnvironmentMessage,
        },
        devices::presence_sensor::{Message as PresenceMessage, PresenceSensorHandler},
        events::door_events::{DerivedDoorEvents, DoorEventsMessage},
        light::{LightHandler, LightHandlerMessage},
        rpc::{self, RpcError},
    },
    settings::workflow::{Combinator, Comparison, Condition, EnvMetric, LeafCondition},
    types::SharedActorState,
    types::db::DoorState,
};
use chrono::{Local, Utc};
use std::time::Duration;

impl From<RpcError> for WorkflowError {
    fn from(e: RpcError) -> Self {
        match e {
            RpcError::ActorNotFound(name) => WorkflowError::ActorNotFound(name),
            RpcError::Messaging(msg) => WorkflowError::Messaging(msg),
        }
    }
}

const QUERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Evaluate a condition against current state. Recursive via `all`/`any`/`not`.
pub async fn eval(state: &SharedActorState, cond: &Condition) -> Result<bool, WorkflowError> {
    match cond {
        Condition::Combinator(c) => eval_combinator(state, c).await,
        Condition::Leaf(l) => eval_leaf(state, l).await,
    }
}

async fn eval_combinator(
    state: &SharedActorState,
    cond: &Combinator,
) -> Result<bool, WorkflowError> {
    match cond {
        Combinator::All(conditions) => {
            for c in conditions {
                if !Box::pin(eval(state, c)).await? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Combinator::Any(conditions) => {
            for c in conditions {
                if Box::pin(eval(state, c)).await? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Combinator::Not(condition) => Ok(!Box::pin(eval(state, condition)).await?),
    }
}

async fn eval_leaf(state: &SharedActorState, cond: &LeafCondition) -> Result<bool, WorkflowError> {
    match cond {
        LeafCondition::Light { ieee_addr, on } => {
            Ok(query_light_on(state.devices.address_or_self(ieee_addr)).await? == *on)
        }
        LeafCondition::Environment {
            sensor,
            metric,
            cmp,
        } => eval_environment(state.devices.address_or_self(sensor), *metric, *cmp).await,
        LeafCondition::Door { ieee_addr, open } => {
            Ok(query_door_open(state.devices.address_or_self(ieee_addr)).await? == *open)
        }
        LeafCondition::Presence { sensor, present } => {
            Ok(query_presence(state.devices.address_or_self(sensor)).await? == *present)
        }
        LeafCondition::TimeOfDay { after, before } => {
            let now = Local::now().time();
            Ok(match (after, before) {
                (Some(a), Some(b)) if a > b => now >= *a || now < *b, // wraps midnight
                (Some(a), Some(b)) => now >= *a && now < *b,
                (Some(a), None) => now >= *a,
                (None, Some(b)) => now < *b,
                (None, None) => true,
            })
        }
        LeafCondition::Sun { is, offset } => {
            Ok(calc::current_period(state.settings.location, Utc::now(), *offset) == *is)
        }
        LeafCondition::GuestMode { active } => Ok(state.workflows.guest_mode().await == *active),
    }
}

async fn query_light_on(ieee_addr: &str) -> Result<bool, WorkflowError> {
    Ok(
        rpc::query_factory(LightHandler::NAME, QUERY_TIMEOUT, |reply| {
            LightHandlerMessage::QueryPowerState {
                ieee_addr: ieee_addr.to_owned(),
                reply,
            }
        })
        .await?,
    )
}

async fn eval_environment(
    sensor: &str,
    metric: EnvMetric,
    cmp: Comparison,
) -> Result<bool, WorkflowError> {
    let reading: Option<LatestReading> =
        rpc::query_factory(EnvironmentSensorHandler::NAME, QUERY_TIMEOUT, |reply| {
            EnvironmentMessage::QueryLatest {
                entity_id: sensor.to_owned(),
                reply,
            }
        })
        .await?;

    let Some(reading) = reading else {
        tracing::warn!("no readings for environment sensor {sensor}");
        return Ok(false);
    };

    let value = match metric {
        EnvMetric::Temperature => Some(reading.temperature),
        EnvMetric::Humidity => reading.humidity,
        EnvMetric::Pressure => reading.pressure,
        EnvMetric::Lux => reading.lux,
        EnvMetric::UvIndex => reading.uv_index,
    };

    let Some(value) = value else {
        tracing::warn!("environment sensor {sensor} has no reading for {metric:?}");
        return Ok(false);
    };

    Ok(cmp.matches(value))
}

async fn query_presence(sensor: &str) -> Result<bool, WorkflowError> {
    let present: Option<bool> =
        rpc::query_factory(PresenceSensorHandler::NAME, QUERY_TIMEOUT, |reply| {
            PresenceMessage::QueryLatest {
                sensor: sensor.to_owned(),
                reply,
            }
        })
        .await?;

    match present {
        Some(present) => Ok(present),
        None => {
            tracing::warn!("no presence reading for sensor {sensor}");
            Ok(false)
        }
    }
}

async fn query_door_open(ieee_addr: &str) -> Result<bool, WorkflowError> {
    let state: Option<DoorState> = rpc::query(DerivedDoorEvents::NAME, QUERY_TIMEOUT, |reply| {
        DoorEventsMessage::QueryState {
            ieee_addr: ieee_addr.to_owned(),
            reply,
        }
    })
    .await?;

    match state {
        Some(state) => Ok(matches!(state, DoorState::Open)),
        None => {
            tracing::warn!("no door state for {ieee_addr}");
            Ok(false)
        }
    }
}
