use crate::{
    mqtt::ZIGBEE2MQTT_BASE,
    settings::IEEEAddress,
    types::SharedActorState,
    zigbee2mqtt::{
        IKEA_LED2201G8::{self},
        Phillips_9290012573A,
    },
};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub enum Entity {
    Phillips9290012573A(Phillips_9290012573A::Phillips9290012573A),
    IKEALED2201G8(IKEA_LED2201G8::IKEALED2201G8),
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
}

pub enum LightHandlerMessage {
    NewEvent(NewEvent),
    TurnOn { ieee_addr: IEEEAddress },
    TurnOff { ieee_addr: IEEEAddress },
    Toggle { ieee_addr: IEEEAddress },
}

pub struct LightHandler {
    shared_actor_state: SharedActorState,
}

impl LightHandler {
    pub const NAME: &str = "light";

    async fn handle(&self, message: LightHandlerMessage) -> Result<(), anyhow::Error> {
        match message {
            LightHandlerMessage::NewEvent(event) => match event.entity {
                Entity::Phillips9290012573A(phillips_9290012573_a) => {
                    sqlx::query!(
                                "INSERT INTO light (event_id, name, ieee_addr, state, brightness) VALUES ($1, $2, $3, $4, $5)",
                                event.event_id,
                                phillips_9290012573_a.device.friendly_name,
                                phillips_9290012573_a.device.ieee_addr,
                                phillips_9290012573_a.state,
                                phillips_9290012573_a.brightness,
                            )
                            .execute(&self.shared_actor_state.db)
                            .await?;
                }
                Entity::IKEALED2201G8(ikealed2201_g8) => {
                    sqlx::query!(
                                "INSERT INTO light (event_id, name, ieee_addr, state, brightness) VALUES ($1, $2, $3, $4, $5)",
                                event.event_id,
                                ikealed2201_g8.device.friendly_name,
                                ikealed2201_g8.device.ieee_addr,
                                ikealed2201_g8.state,
                                ikealed2201_g8.brightness,
                            )
                            .execute(&self.shared_actor_state.db)
                            .await?;
                }
            },
            LightHandlerMessage::TurnOn { ieee_addr } => {
                self.send_mqtt_state(ieee_addr, serde_json::json!({"state": "ON"}))
                    .await?;
            }
            LightHandlerMessage::TurnOff { ieee_addr } => {
                self.send_mqtt_state(ieee_addr, serde_json::json!({"state": "OFF"}))
                    .await?;
            }
            LightHandlerMessage::Toggle { ieee_addr } => {
                self.send_mqtt_state(ieee_addr, serde_json::json!({"state": "TOGGLE"}))
                    .await?;
            }
        }

        Ok(())
    }

    async fn send_mqtt_state(
        &self,
        ieee_addr: String,
        state: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        let friendly_name = {
            let devices_map = self.shared_actor_state.known_devices_map.read().await;
            devices_map.get(&ieee_addr).cloned()
        };

        let Some(friendly_name) = friendly_name else {
            tracing::warn!("could not find device for {ieee_addr}");
            return Ok(());
        };

        let topic = format!("{ZIGBEE2MQTT_BASE}/{friendly_name}/set");
        self.shared_actor_state
            .mqtt
            .send_event(topic, state)
            .await?;

        Ok(())
    }
}

impl Worker for LightHandler {
    type Key = ();
    type Message = LightHandlerMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), LightHandlerMessage>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    #[tracing::instrument(name = "light", skip(self, _wid, _factory, msg, _state))]
    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), LightHandlerMessage>>,
        Job { msg, .. }: Job<(), LightHandlerMessage>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Err(e) = Self::handle(self, msg).await {
            tracing::error!("error while handling message: {e}")
        }

        Ok(())
    }
}

pub struct LightHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}
impl WorkerBuilder<LightHandler, ()> for LightHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (LightHandler, ()) {
        (
            LightHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
