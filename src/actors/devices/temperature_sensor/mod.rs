
use crate::{
    types::SharedActorState,
    zigbee2mqtt::{Aqara_WSDCGQ12LM, IKEA_E2112, Lumi_WSDCGQ11LM},
};
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
                    let id = self
                        .shared_actor_state
                        .settings
                        .temperature_sensors
                        .get(&aqara_wsdcgq12_lm.device.ieee_addr)
                        .map(|s| &s.id);

                    sqlx::query!(
                        "INSERT INTO temperature_sensor (event_id, id, name, ieee_addr, temperature, battery, humidity, pressure) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                        event.event_id,
                        id,
                        aqara_wsdcgq12_lm.device.friendly_name,
                        aqara_wsdcgq12_lm.device.ieee_addr,
                        aqara_wsdcgq12_lm.temperature,
                        aqara_wsdcgq12_lm.battery,
                        aqara_wsdcgq12_lm.humidity,
                        aqara_wsdcgq12_lm.pressure,
                    ).execute(&self.shared_actor_state.db).await?;
                }
                Entity::LumiWSDCGQ11LM(lumi_wsdcgq11_lm) => {
                    let id = self
                        .shared_actor_state
                        .settings
                        .temperature_sensors
                        .get(&lumi_wsdcgq11_lm.device.ieee_addr)
                        .map(|s| &s.id);

                    sqlx::query!(
                        "INSERT INTO temperature_sensor (event_id, id, name, ieee_addr, temperature, battery, humidity, pressure) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                        event.event_id,
                        id,
                        lumi_wsdcgq11_lm.device.friendly_name,
                        lumi_wsdcgq11_lm.device.ieee_addr,
                        lumi_wsdcgq11_lm.temperature,
                        lumi_wsdcgq11_lm.battery,
                        lumi_wsdcgq11_lm.humidity,
                        lumi_wsdcgq11_lm.pressure,
                    ).execute(&self.shared_actor_state.db).await?;
                }
                Entity::IKEAE2112(ikea_e2112) => {
                    let id = self
                        .shared_actor_state
                        .settings
                        .temperature_sensors
                        .get(&ikea_e2112.device.ieee_addr)
                        .map(|s| &s.id);

                    sqlx::query!(
                        "INSERT INTO temperature_sensor (event_id, id, name, ieee_addr, temperature, pm25, humidity, voc_index) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                        event.event_id,
                        id,
                        ikea_e2112.device.friendly_name,
                        ikea_e2112.device.ieee_addr,
                        ikea_e2112.temperature as f64,
                        ikea_e2112.pm25,
                        ikea_e2112.humidity as f64,
                        ikea_e2112.voc_index
                    ).execute(&self.shared_actor_state.db).await?;
                }
            },
        }

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
