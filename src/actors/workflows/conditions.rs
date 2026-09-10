//! Condition evaluation, shared between two callers:
//! - workflow step `when` guards ([`super::WorkflowWorker`]), and
//! - trigger `when` gates ([`crate::actors::workflows::dispatcher`]).
//!
//! Both need to answer the same boolean predicates against live device/sensor
//! state, so the actor-query RPC logic lives here once and takes a plain
//! [`AppState`] rather than being tied to the workflow worker.

use super::WorkflowError;
use crate::actors::sun::calc;
use crate::actors::workflows::manager::WorkflowManager;
use crate::{
    actors::{
        devices::door_events::{DerivedDoorEvents, DoorEventsMessage},
        devices::environment_sensor::{
            EnvironmentSensorHandler, LatestReading, Message as EnvironmentMessage,
        },
        devices::light::{LightHandler, LightHandlerMessage},
        devices::presence_sensor::{Message as PresenceMessage, PresenceSensorHandler},
        integrations::solar::{SolarActor, SolarMessage},
        system::rpc::{self, RpcError},
    },
    db::DoorState,
    event_bus::{ForecastDay, SolarMetric, WeatherMetric, WeatherReading, WeatherSource},
    integrations::{home_assistant::HomeAssistant, solar, willyweather::WillyWeather},
    settings::switch_metric::SwitchMetric,
    settings::workflow::{Combinator, Comparison, Condition, EnvMetric, LeafCondition},
    state::AppState,
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
pub async fn eval(state: &AppState, cond: &Condition) -> Result<bool, WorkflowError> {
    match cond {
        Condition::Combinator(c) => eval_combinator(state, c).await,
        Condition::Leaf(l) => eval_leaf(state, l).await,
    }
}

async fn eval_combinator(state: &AppState, cond: &Combinator) -> Result<bool, WorkflowError> {
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

async fn eval_leaf(state: &AppState, cond: &LeafCondition) -> Result<bool, WorkflowError> {
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
        LeafCondition::Mode { mode, active } => Ok(state
            .handles
            .expect::<WorkflowManager>()
            .mode_active(*mode)
            .await
            == *active),
        LeafCondition::Solar { metric, cmp } => eval_solar(state, *metric, *cmp).await,
        LeafCondition::HomeAssistant {
            entity_id,
            state: expected,
        } => eval_home_assistant(state, entity_id, expected).await,
        LeafCondition::SmartSwitch {
            ieee_addr,
            metric,
            cmp,
        } => {
            eval_smart_switch(
                state,
                state.devices.address_or_self(ieee_addr),
                *metric,
                *cmp,
            )
            .await
        }
        LeafCondition::Weather {
            source,
            metric,
            day,
            cmp,
        } => eval_weather(state, *source, *metric, *day, *cmp).await,
    }
}

async fn eval_weather(
    state: &AppState,
    source: WeatherSource,
    metric: WeatherMetric,
    day: Option<ForecastDay>,
    cmp: Comparison,
) -> Result<bool, WorkflowError> {
    let value = match (source, day) {
        (WeatherSource::Bom, _) => {
            let readings: Vec<WeatherReading> =
                rpc::query(SolarActor::NAME, QUERY_TIMEOUT, |reply| {
                    SolarMessage::LatestWeather { reply }
                })
                .await?;

            readings
                .iter()
                .find(|reading| reading.metric == metric)
                .map(|reading| reading.value)
        }
        (WeatherSource::WillyWeather, Some(day)) => {
            let forecast = state
                .handles
                .expect::<WillyWeather>()
                .forecast(&state.settings.willyweather.default_location)
                .await
                .map_err(anyhow::Error::from)?;

            forecast
                .days
                .get(day.index())
                .and_then(|details| match metric {
                    WeatherMetric::MaxTemp => Some(details.max as f64),
                    WeatherMetric::MinTemp => Some(details.min as f64),
                    WeatherMetric::UvMax => details.uv,
                    _ => None,
                })
        }
        (WeatherSource::WillyWeather, None) => None,
    };

    let Some(value) = value else {
        tracing::warn!(
            "no {} weather reading for {}",
            source.as_str(),
            metric.var_name(day)
        );
        return Ok(false);
    };

    Ok(cmp.matches(value))
}

async fn eval_solar(
    state: &AppState,
    metric: SolarMetric,
    cmp: Comparison,
) -> Result<bool, WorkflowError> {
    let value = match metric {
        SolarMetric::Current => solar::queries::current_wh(&state.db)
            .await
            .map_err(anyhow::Error::from)?,
        SolarMetric::Avg15m | SolarMetric::Avg1h | SolarMetric::Avg3h => {
            let averages = solar::queries::statistics(&state.db)
                .await
                .map_err(anyhow::Error::from)?
                .averages;

            match metric {
                SolarMetric::Avg15m => averages.last_15_mins,
                SolarMetric::Avg1h => averages.last_1_hour,
                _ => averages.last_3_hours,
            }
        }
    };

    let Some(value) = value else {
        tracing::warn!("no solar reading for {}", metric.var_name());
        return Ok(false);
    };

    Ok(cmp.matches(value))
}

async fn eval_home_assistant(
    state: &AppState,
    entity_id: &str,
    expected: &str,
) -> Result<bool, WorkflowError> {
    let Some(home_assistant) = state.handles.get::<HomeAssistant>() else {
        return Err(WorkflowError::HomeAssistantNotConfigured);
    };

    let entity = home_assistant.get_state(entity_id).await?;

    let Some(current) = entity.get("state").and_then(|s| s.as_str()) else {
        tracing::warn!("home assistant entity {entity_id} has no state");
        return Ok(false);
    };

    Ok(current == expected)
}

async fn eval_smart_switch(
    state: &AppState,
    ieee_addr: &str,
    metric: SwitchMetric,
    cmp: Comparison,
) -> Result<bool, WorkflowError> {
    let latest = state
        .repos
        .smart_switch()
        .latest(ieee_addr)
        .await
        .map_err(anyhow::Error::from)?;

    let Some(latest) = latest else {
        tracing::warn!("no readings for smart switch {ieee_addr}");
        return Ok(false);
    };

    let value = match metric {
        SwitchMetric::Power => latest.power as f64,
        SwitchMetric::Voltage => latest.voltage as f64,
        SwitchMetric::Current => latest.current,
        SwitchMetric::Energy => latest.energy,
    };

    Ok(cmp.matches(value))
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
