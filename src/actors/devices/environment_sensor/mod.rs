use crate::{
    esphome,
    event_bus::{EventBusMessage, SensorReading},
    types::SharedActorState,
    zigbee2mqtt::{Aqara_WSDCGQ12LM, IKEA_E2112, Lumi_WSDCGQ11LM},
};
use chrono::Utc;
use ractor::{
    ActorProcessingErr, ActorRef, RpcReplyPort,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use std::collections::HashMap;
use uuid::Uuid;

pub mod spawn;

pub enum Entity {
    AqaraWSDCGQ12LM(Aqara_WSDCGQ12LM::AqaraWSDCGQ12LM),
    LumiWSDCGQ11LM(Lumi_WSDCGQ11LM::LumiWSDCGQ11LM),
    IKEAE2112(IKEA_E2112::IKEAE2112),
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
    shared_actor_state: SharedActorState,
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
                let row = sqlx::query!(
                    "SELECT temperature, humidity, pressure, lux, uv_index \
                     FROM latest_temperature_sensor WHERE entity_id = $1",
                    entity_id
                )
                .fetch_optional(&self.shared_actor_state.db)
                .await?;

                let reading = row.map(|r| LatestReading {
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
                Entity::AqaraWSDCGQ12LM(aqara_wsdcgq12_lm) => {
                    self.save_environment_details(
                        event.event_id,
                        aqara_wsdcgq12_lm.device.friendly_name,
                        aqara_wsdcgq12_lm.device.ieee_addr,
                        aqara_wsdcgq12_lm.temperature,
                        Some(aqara_wsdcgq12_lm.battery),
                        Some(aqara_wsdcgq12_lm.humidity),
                        Some(aqara_wsdcgq12_lm.pressure),
                        None,
                        None,
                        None,
                        None,
                    )
                    .await?;
                }
                Entity::LumiWSDCGQ11LM(lumi_wsdcgq11_lm) => {
                    self.save_environment_details(
                        event.event_id,
                        lumi_wsdcgq11_lm.device.friendly_name,
                        lumi_wsdcgq11_lm.device.ieee_addr,
                        lumi_wsdcgq11_lm.temperature,
                        Some(lumi_wsdcgq11_lm.battery),
                        Some(lumi_wsdcgq11_lm.humidity),
                        Some(lumi_wsdcgq11_lm.pressure),
                        None,
                        None,
                        None,
                        None,
                    )
                    .await?;
                }
                Entity::IKEAE2112(ikea_e2112) => {
                    self.save_environment_details(
                        event.event_id,
                        ikea_e2112.device.friendly_name,
                        ikea_e2112.device.ieee_addr,
                        ikea_e2112.temperature as f64,
                        None,
                        Some(ikea_e2112.humidity as f64),
                        None,
                        Some(ikea_e2112.pm25),
                        Some(ikea_e2112.voc_index),
                        None,
                        None,
                    )
                    .await?;
                }
                Entity::Esphome {
                    node,
                    object_id,
                    value,
                } => {
                    let readings = state.esphome_readings.entry(node.clone()).or_default();
                    match object_id.as_str() {
                        esphome::DPS310_TEMPERATURE_OBJECT_ID
                        | esphome::AIR_TEMPERATURE_OBJECT_ID
                        | esphome::SHTC3_TEMPERATURE_OBJECT_ID => {
                            readings.temperature = Some(value)
                        }
                        esphome::AIR_HUMIDITY_OBJECT_ID | esphome::SHTC3_HUMIDITY_OBJECT_ID => {
                            readings.humidity = Some(value)
                        }
                        esphome::DPS310_PRESSURE_OBJECT_ID => readings.pressure = Some(value),
                        esphome::LTR390_LIGHT_OBJECT_ID
                        | esphome::BH1750_ILLUMINANCE_OBJECT_ID => readings.lux = Some(value),
                        esphome::LTR390_UV_INDEX_OBJECT_ID => readings.uv_index = Some(value),
                        other => {
                            // not a temperature-sensor entity (e.g. soil_moisture); ignore
                            tracing::debug!("ignoring esphome sensor entity: {other}");
                            return Ok(());
                        }
                    }

                    // temperature is the required column; without it there is nothing to store yet
                    let Some(temperature) = readings.temperature else {
                        return Ok(());
                    };
                    let humidity = readings.humidity;
                    let pressure = readings.pressure;
                    let lux = readings.lux;
                    let uv_index = readings.uv_index;

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
                        None,
                        None,
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
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO temperature_sensor (event_id, id, name, ieee_addr, temperature, battery, humidity, pressure, pm25, voc_index, lux, uv_index) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            event_id,
            id,
            friendly_name,
            ieee_addr,
            temperature,
            battery,
            humidity,
            pressure,
            pm25,
            voc_index,
            lux,
            uv_index,
        ).execute(&self.shared_actor_state.db).await?;

        sqlx::query!(
            r#"INSERT INTO latest_temperature_sensor (entity_id, name, ieee_addr, temperature, battery, humidity, pressure, pm25, voc_index, lux, uv_index, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (entity_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                ieee_addr = EXCLUDED.ieee_addr,
                temperature = EXCLUDED.temperature,
                battery = EXCLUDED.battery,
                humidity = EXCLUDED.humidity,
                pressure = EXCLUDED.pressure,
                pm25 = EXCLUDED.pm25,
                voc_index = EXCLUDED.voc_index,
                lux = EXCLUDED.lux,
                uv_index = EXCLUDED.uv_index,
                updated_at = EXCLUDED.updated_at
            "#,
            id,
            friendly_name,
            ieee_addr,
            temperature,
            battery,
            humidity,
            pressure,
            pm25,
            voc_index,
            lux,
            uv_index,
            now,
        ).execute(&self.shared_actor_state.db).await?;

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

impl Worker for EnvironmentSensorHandler {
    type Key = ();
    type Message = Message;
    type State = EnvironmentSensorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(EnvironmentSensorState::default())
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        Job { msg, .. }: Job<(), Message>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg, state).await {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct EnvironmentSensorHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}
impl WorkerBuilder<EnvironmentSensorHandler, ()> for EnvironmentSensorHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (EnvironmentSensorHandler, ()) {
        (
            EnvironmentSensorHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
