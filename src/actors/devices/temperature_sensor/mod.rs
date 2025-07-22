use std::collections::HashMap;

use crate::{
    settings::{IEEEAddress, TemperatureSensorSettings},
    types::SharedActorState,
    zigbee2mqtt::{Aqara_WSDCGQ12LM, Lumi_WSDCGQ11LM},
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
    temperature_sensor_settings: HashMap<IEEEAddress, TemperatureSensorSettings>,
}

impl TemperatureSensorHandler {
    pub const NAME: &str = "temperature-sensor";

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::NewEvent(event) => match event.entity {
                Entity::AqaraWSDCGQ12LM(aqara_wsdcgq12_lm) => {
                    let id = self
                        .temperature_sensor_settings
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
                        .temperature_sensor_settings
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
    pub temperature_sensor_settings: HashMap<IEEEAddress, TemperatureSensorSettings>,
}
impl WorkerBuilder<TemperatureSensorHandler, ()> for TemperatureSensorHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (TemperatureSensorHandler, ()) {
        (
            TemperatureSensorHandler {
                shared_actor_state: self.shared_actor_state.clone(),
                temperature_sensor_settings: self.temperature_sensor_settings.clone(),
            },
            (),
        )
    }
}
