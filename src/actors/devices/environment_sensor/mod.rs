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

pub enum Entity {
    Zigbee {
        address: String,
        friendly_name: String,
        readings: Vec<(Metric, f64)>,
        battery: Option<i64>,
    },
    Esphome {
        node: String,
        object_id: String,
        value: f64,
    },
}

#[derive(Default)]
struct EsphomeReadings {
    temperature: Option<f64>,
    humidity: Option<f64>,
    pressure: Option<f64>,
    lux: Option<f64>,
    uv_index: Option<f64>,
    pm25: Option<f64>,
    voc_index: Option<f64>,
}

#[derive(Default)]
pub struct EnvironmentSensorState {
    esphome_readings: HashMap<String, EsphomeReadings>,
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
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
            Message::NewEvent(event) => match event.entity {
                Entity::Zigbee {
                    address,
                    friendly_name,
                    readings,
                    battery,
                } => {
                    let reading = |wanted: Metric| {
                        readings
                            .iter()
                            .find(|(metric, _)| *metric == wanted)
                            .map(|(_, value)| *value)
                    };

                    let Some(temperature) = reading(Metric::Temperature) else {
                        tracing::debug!(
                            "ignoring zigbee environment payload without temperature: {address}"
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
                Entity::Esphome {
                    node,
                    object_id,
                    value,
                } => {
                    let Some(metric) = self
                        .shared_actor_state
                        .devices
                        .environment(&node)
                        .and_then(|s| s.entities.get(&object_id).copied())
                    else {
                        tracing::debug!("ignoring esphome sensor entity: {object_id}");
                        return Ok(());
                    };

                    let readings = state.esphome_readings.entry(node.clone()).or_default();
                    match metric {
                        Metric::Temperature => readings.temperature = Some(value),
                        Metric::Humidity => readings.humidity = Some(value),
                        Metric::Pressure => readings.pressure = Some(value),
                        Metric::Lux => readings.lux = Some(value),
                        Metric::UvIndex => readings.uv_index = Some(value),
                        Metric::Pm25 => readings.pm25 = Some(value),
                        Metric::VocIndex => readings.voc_index = Some(value),
                    }

                    // temperature is the required column; without it there is nothing to store yet
                    let Some(temperature) = readings.temperature else {
                        return Ok(());
                    };
                    let humidity = readings.humidity;
                    let pressure = readings.pressure;
                    let lux = readings.lux;
                    let uv_index = readings.uv_index;
                    let pm25 = readings.pm25.map(|v| v.round() as i64);
                    let voc_index = readings.voc_index.map(|v| v.round() as i64);

                    let friendly_name = self
                        .shared_actor_state
                        .devices
                        .friendly_name(&node)
                        .await
                        .unwrap_or_else(|| node.clone());

                    self.save_environment_details(
                        event.event_id,
                        friendly_name,
                        node,
                        temperature,
                        None,
                        humidity,
                        pressure,
                        pm25,
                        voc_index,
                        lux,
                        uv_index,
                    )
                    .await?;
                }
            },
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

    async fn handle(&self, message: Self::Message, state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message, state).await
    }
}
