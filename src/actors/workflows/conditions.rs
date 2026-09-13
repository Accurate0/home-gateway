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
use crate::lua::LuaCallContext;
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
    integrations::{home_assistant::HomeAssistant, solar},
    settings::workflow::{
        Combinator, CompareOp, Comparison, Condition, EnvMetric, LeafCondition, SwitchMetric,
    },
    state::AppState,
    templating::{Expr, Literal},
    variables::Vars,
};
use chrono::{Local, Utc};
use std::time::Duration;
use uuid::Uuid;

impl From<RpcError> for WorkflowError {
    fn from(e: RpcError) -> Self {
        match e {
            RpcError::ActorNotFound(name) => WorkflowError::ActorNotFound(name),
            RpcError::Messaging(msg) => WorkflowError::Messaging(msg),
        }
    }
}

/// Evaluate a condition against current state. Recursive via `all`/`any`/`not`.
pub async fn eval(state: &AppState, vars: &Vars, cond: &Condition) -> Result<bool, WorkflowError> {
    match cond {
        Condition::Combinator(c) => eval_combinator(state, vars, c).await,
        Condition::Leaf(l) => eval_leaf(state, vars, l).await,
    }
}

async fn eval_combinator(
    state: &AppState,
    vars: &Vars,
    cond: &Combinator,
) -> Result<bool, WorkflowError> {
    match cond {
        Combinator::All(conditions) => {
            for c in conditions {
                if !Box::pin(eval(state, vars, c)).await? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Combinator::Any(conditions) => {
            for c in conditions {
                if Box::pin(eval(state, vars, c)).await? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Combinator::Not(condition) => Ok(!Box::pin(eval(state, vars, condition)).await?),
    }
}

fn eval_var(
    vars: &Vars,
    var: &Expr,
    op: CompareOp,
    expected: &Literal,
) -> Result<bool, WorkflowError> {
    let actual = var.value(vars).map_err(WorkflowError::Template)?;
    let expected = expected.to_value();

    let matched = match (actual.as_f64(), expected.as_f64()) {
        (Some(actual), Some(expected)) => Comparison {
            op,
            value: expected,
        }
        .matches(actual),
        _ => matches!(op, CompareOp::Eq) && actual == expected,
    };

    Ok(matched)
}

async fn eval_leaf(
    state: &AppState,
    vars: &Vars,
    cond: &LeafCondition,
) -> Result<bool, WorkflowError> {
    let timeout = state.settings.workflow.condition_timeout();

    match cond {
        LeafCondition::Light { ieee_addr, on } => {
            Ok(query_light_on(state.devices.address_or_self(ieee_addr), timeout).await? == *on)
        }
        LeafCondition::Environment {
            sensor,
            metric,
            cmp,
        } => {
            eval_environment(
                state.devices.address_or_self(sensor),
                *metric,
                *cmp,
                timeout,
            )
            .await
        }
        LeafCondition::Door { ieee_addr, open } => {
            Ok(query_door_open(state.devices.address_or_self(ieee_addr), timeout).await? == *open)
        }
        LeafCondition::Presence { sensor, present } => {
            Ok(query_presence(state.devices.address_or_self(sensor), timeout).await? == *present)
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
        LeafCondition::Mode { is } => Ok(state
            .handles
            .expect::<WorkflowManager>()
            .current_mode()
            .await
            == *is),
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
        LeafCondition::Var { var, op, value } => eval_var(vars, var, *op, value),
        LeafCondition::Lua { script } => {
            let cx = LuaCallContext::new(state.clone(), Uuid::nil(), "condition");

            Ok(state.lua.run_bool(&cx, script, vars).await?)
        }
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
            let readings: Vec<WeatherReading> = rpc::query(
                SolarActor::NAME,
                state.settings.workflow.condition_timeout(),
                |reply| SolarMessage::LatestWeather { reply },
            )
            .await?;

            readings
                .iter()
                .find(|reading| reading.metric == metric)
                .map(|reading| reading.value)
        }
        (WeatherSource::WillyWeather, Some(day)) => {
            let forecast = state
                .repos
                .willyweather()
                .forecast(&state.settings.willyweather.default_location)
                .await
                .map_err(anyhow::Error::from)?;

            forecast.and_then(|forecast| {
                forecast
                    .days
                    .get(day.index())
                    .and_then(|details| details.metric(metric))
            })
        }
        (WeatherSource::WillyWeather, None) => None,
    };

    let Some(value) = value else {
        tracing::warn!(
            "no {} weather reading for {}",
            source.as_str(),
            metric.label(day)
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
        tracing::warn!("no solar reading for {metric}");
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

pub async fn query_light_on(ieee_addr: &str, timeout: Duration) -> Result<bool, WorkflowError> {
    Ok(rpc::query_factory(LightHandler::NAME, timeout, |reply| {
        LightHandlerMessage::QueryPowerState {
            ieee_addr: ieee_addr.to_owned(),
            reply,
        }
    })
    .await?)
}

pub async fn query_environment(
    sensor: &str,
    timeout: Duration,
) -> Result<Option<LatestReading>, WorkflowError> {
    Ok(
        rpc::query_factory(EnvironmentSensorHandler::NAME, timeout, |reply| {
            EnvironmentMessage::QueryLatest {
                entity_id: sensor.to_owned(),
                reply,
            }
        })
        .await?,
    )
}

pub fn environment_metric(reading: &LatestReading, metric: EnvMetric) -> Option<f64> {
    match metric {
        EnvMetric::Temperature => Some(reading.temperature),
        EnvMetric::Humidity => reading.humidity,
        EnvMetric::Pressure => reading.pressure,
        EnvMetric::Lux => reading.lux,
        EnvMetric::UvIndex => reading.uv_index,
    }
}

async fn eval_environment(
    sensor: &str,
    metric: EnvMetric,
    cmp: Comparison,
    timeout: Duration,
) -> Result<bool, WorkflowError> {
    let reading = query_environment(sensor, timeout).await?;

    let Some(reading) = reading else {
        tracing::warn!("no readings for environment sensor {sensor}");
        return Ok(false);
    };

    let Some(value) = environment_metric(&reading, metric) else {
        tracing::warn!("environment sensor {sensor} has no reading for {metric:?}");
        return Ok(false);
    };

    Ok(cmp.matches(value))
}

pub async fn query_presence(sensor: &str, timeout: Duration) -> Result<bool, WorkflowError> {
    let present: Option<bool> = rpc::query_factory(PresenceSensorHandler::NAME, timeout, |reply| {
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

pub async fn query_door_open(ieee_addr: &str, timeout: Duration) -> Result<bool, WorkflowError> {
    let state: Option<DoorState> = rpc::query(DerivedDoorEvents::NAME, timeout, |reply| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variables::{Node, Value};

    fn vars() -> Vars {
        let mut event = Node::empty();
        event.insert("new_price", Node::Value(Some(Value::Float(2.5))));
        event.insert("name", Node::Value(Some(Value::String("Milk".to_owned()))));
        event.insert("percent", Node::Value(None));

        Vars::default().with("event", event)
    }

    fn expr(raw: &str) -> Expr {
        Expr::parse(raw).expect("valid expression")
    }

    #[test]
    fn var_leaf_compares_numbers_across_int_and_float() {
        assert!(
            eval_var(
                &vars(),
                &expr("event.new_price"),
                CompareOp::Lt,
                &Literal::Int(3)
            )
            .unwrap()
        );
        assert!(
            !eval_var(
                &vars(),
                &expr("event.new_price"),
                CompareOp::Gte,
                &Literal::Float(2.6)
            )
            .unwrap()
        );
    }

    #[test]
    fn var_leaf_compares_strings_for_equality() {
        let milk = Literal::String("Milk".to_owned());

        assert!(eval_var(&vars(), &expr("event.name"), CompareOp::Eq, &milk).unwrap());
        assert!(!eval_var(&vars(), &expr("event.name"), CompareOp::Gt, &milk).unwrap());
    }

    #[test]
    fn var_leaf_uses_the_default_for_a_missing_value() {
        assert!(
            eval_var(
                &vars(),
                &expr("event.percent | default(100)"),
                CompareOp::Gt,
                &Literal::Int(50)
            )
            .unwrap()
        );
        assert!(
            eval_var(
                &vars(),
                &expr("event.percent"),
                CompareOp::Gt,
                &Literal::Int(50)
            )
            .is_err()
        );
    }
}
