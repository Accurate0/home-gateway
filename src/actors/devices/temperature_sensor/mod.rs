use crate::{
    types::SharedActorState,
    zigbee2mqtt::{Aqara_WSDCGQ12LM, IKEA_E2112, Lumi_WSDCGQ11LM},
};
use chrono::Utc;
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub enum Entity {
    AqaraWSDCGQ12LM(Aqara_WSDCGQ12LM::AqaraWSDCGQ12LM),
    LumiWSDCGQ11LM(Lumi_WSDCGQ11LM::LumiWSDCGQ11LM),
    IKEAE2112(IKEA_E2112::IKEAE2112),
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
}

pub enum Message {
    NewEvent(NewEvent),
}

pub struct TemperatureSensorHandler {
    shared_actor_state: SharedActorState,
}

impl TemperatureSensorHandler {
    pub const NAME: &str = "temperature-sensor";

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::NewEvent(event) => match event.entity {
                Entity::AqaraWSDCGQ12LM(aqara_wsdcgq12_lm) => {
                    self.save_temperature_details(
                        event.event_id,
                        aqara_wsdcgq12_lm.device.friendly_name,
                        aqara_wsdcgq12_lm.device.ieee_addr,
                        aqara_wsdcgq12_lm.temperature,
                        Some(aqara_wsdcgq12_lm.battery),
                        aqara_wsdcgq12_lm.humidity,
                        Some(aqara_wsdcgq12_lm.pressure),
                        None,
                        None,
                    )
                    .await?;
                }
                Entity::LumiWSDCGQ11LM(lumi_wsdcgq11_lm) => {
                    self.save_temperature_details(
                        event.event_id,
                        lumi_wsdcgq11_lm.device.friendly_name,
                        lumi_wsdcgq11_lm.device.ieee_addr,
                        lumi_wsdcgq11_lm.temperature,
                        Some(lumi_wsdcgq11_lm.battery),
                        lumi_wsdcgq11_lm.humidity,
                        Some(lumi_wsdcgq11_lm.pressure),
                        None,
                        None,
                    )
                    .await?;
                }
                Entity::IKEAE2112(ikea_e2112) => {
                    self.save_temperature_details(
                        event.event_id,
                        ikea_e2112.device.friendly_name,
                        ikea_e2112.device.ieee_addr,
                        ikea_e2112.temperature as f64,
                        None,
                        ikea_e2112.humidity as f64,
                        None,
                        Some(ikea_e2112.pm25),
                        Some(ikea_e2112.voc_index),
                    )
                    .await?;
                }
            },
        }

        Ok(())
    }

    async fn save_temperature_details(
        &self,
        event_id: Uuid,
        friendly_name: String,
        ieee_addr: String,
        temperature: f64,
        battery: Option<i64>,
        humidity: f64,
        pressure: Option<f64>,
        pm25: Option<i64>,
        voc_index: Option<i64>,
    ) -> Result<(), anyhow::Error> {
        let settings = self.shared_actor_state.settings.load();
        let id = settings.temperature_sensors.get(&ieee_addr).map(|s| &s.id);
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO temperature_sensor (event_id, id, name, ieee_addr, temperature, battery, humidity, pressure, pm25, voc_index) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
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
        ).execute(&self.shared_actor_state.db).await?;

        sqlx::query!(
            r#"INSERT INTO latest_temperature_sensor (entity_id, name, ieee_addr, temperature, battery, humidity, pressure, pm25, voc_index, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
            now,
        ).execute(&self.shared_actor_state.db).await?;

        Ok(())
    }
}

impl Worker for TemperatureSensorHandler {
    type Key = ();
    type Message = Message;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    #[tracing::instrument(name = "temperature-sensor", skip(self, _wid, _factory, msg, _state))]
    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), Message>>,
        Job { msg, .. }: Job<(), Message>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg).await {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct TemperatureSensorHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}
impl WorkerBuilder<TemperatureSensorHandler, ()> for TemperatureSensorHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (TemperatureSensorHandler, ()) {
        (
            TemperatureSensorHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
