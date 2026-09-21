pub mod lua;

use crate::actors::devices::handler::DeviceHandler;
use crate::repo::environment::EnvironmentReading;
use crate::{
    event_bus::{EventBusMessage, SensorReading},
    settings::Metric,
    state::AppState,
};
use ractor::RpcReplyPort;
use std::collections::HashMap;
use uuid::Uuid;

pub struct Entity {
    pub address: String,
    pub friendly_name: String,
    pub readings: Vec<(Metric, f64)>,
    pub battery: Option<i64>,
}

#[derive(Default)]
pub struct EnvironmentSensorState {
    latest: HashMap<String, HashMap<Metric, f64>>,
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
    pub traceparent: crate::tracing_context::TraceParent,
}

/// Latest persisted readings for an environment sensor, used to answer workflow
/// condition queries without the workflow worker touching the database directly.
pub struct LatestReading {
    pub temperature: f64,
    pub humidity: Option<f64>,
    pub pressure: Option<f64>,
    pub lux: Option<f64>,
    pub uv_index: Option<f64>,
}

pub enum Message {
    NewEvent(Box<NewEvent>),
    QueryLatest {
        entity_id: String,
        reply: RpcReplyPort<Option<LatestReading>>,
    },
}

impl crate::tracing_context::TracedMessage for Message {
    fn traceparent(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => event.traceparent.as_deref(),
            Message::QueryLatest { .. } => None,
        }
    }

    fn subject(&self) -> Option<&str> {
        match self {
            Message::NewEvent(event) => Some(&event.entity.address),
            Message::QueryLatest { entity_id, .. } => Some(entity_id),
        }
    }
}

pub struct EnvironmentSensorHandler {
    shared_actor_state: AppState,
}

impl EnvironmentSensorHandler {
    pub const NAME: &str = "environment-sensor";

    async fn handle(
        &self,
        message: Message,
        state: &mut EnvironmentSensorState,
    ) -> Result<(), anyhow::Error> {
        match message {
            Message::QueryLatest { entity_id, reply } => {
                let reading = self
                    .shared_actor_state
                    .repos
                    .environment()
                    .latest(&entity_id)
                    .await?
                    .map(|r| LatestReading {
                        temperature: r.temperature,
                        humidity: r.humidity,
                        pressure: r.pressure,
                        lux: r.lux,
                        uv_index: r.uv_index,
                    });

                reply.send(reading)?;
                return Ok(());
            }
            Message::NewEvent(event) => {
                let Entity {
                    address,
                    friendly_name,
                    readings,
                    battery,
                } = event.entity;

                let latest = state.latest.entry(address.clone()).or_default();
                latest.extend(readings);

                let reading = |metric: Metric| latest.get(&metric).copied();

                let Some(temperature) = reading(Metric::Temperature) else {
                    tracing::debug!(
                        "holding environment reading for {address} until it reports temperature"
                    );
                    return Ok(());
                };

                self.save_environment_details(
                    event.event_id,
                    friendly_name,
                    address,
                    temperature,
                    battery,
                    reading(Metric::Humidity),
                    reading(Metric::Pressure),
                    reading(Metric::Pm25).map(|v| v.round() as i64),
                    reading(Metric::VocIndex).map(|v| v.round() as i64),
                    reading(Metric::Lux),
                    reading(Metric::UvIndex),
                )
                .await?;
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn save_environment_details(
        &self,
        event_id: Uuid,
        friendly_name: String,
        ieee_addr: String,
        temperature: f64,
        battery: Option<i64>,
        humidity: Option<f64>,
        pressure: Option<f64>,
        pm25: Option<i64>,
        voc_index: Option<i64>,
        lux: Option<f64>,
        uv_index: Option<f64>,
    ) -> Result<(), anyhow::Error> {
        let id = self
            .shared_actor_state
            .devices
            .environment(&ieee_addr)
            .map(|s| &s.id);
        self.shared_actor_state
            .repos
            .environment()
            .record(&EnvironmentReading {
                event_id,
                id: id.cloned(),
                friendly_name: friendly_name.clone(),
                ieee_addr: ieee_addr.clone(),
                temperature,
                battery,
                humidity,
                pressure,
                pm25,
                voc_index,
                lux,
                uv_index,
            })
            .await?;

        // publish all present readings in a single event, keyed by device id, so
        // `environment` triggers and subscribers see the full metric snapshot
        let readings: Vec<SensorReading> = [
            Some(SensorReading::Temperature { value: temperature }),
            humidity.map(|value| SensorReading::Humidity { value }),
            pressure.map(|value| SensorReading::Pressure { value }),
            lux.map(|value| SensorReading::Lux { value }),
            uv_index.map(|value| SensorReading::UvIndex { value }),
            pm25.map(|value| SensorReading::Pm25 {
                value: value as f64,
            }),
            voc_index.map(|value| SensorReading::VocIndex {
                value: value as f64,
            }),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.shared_actor_state
            .event_bus
            .publish(EventBusMessage::Environment {
                event_id,
                sensor: ieee_addr,
                readings,
            });

        Ok(())
    }
}

impl DeviceHandler for EnvironmentSensorHandler {
    const NAME: &'static str = EnvironmentSensorHandler::NAME;

    type Message = Message;
    type State = EnvironmentSensorState;

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    fn workers(workers: &crate::settings::ActorWorkerSettings) -> usize {
        workers.environment_sensor
    }

    async fn handle(&self, message: Self::Message, state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message, state).await
    }
}
