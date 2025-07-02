use crate::{
    actors::events::door_events::{DoorEvents, DoorEventsSupervisor},
    types::SharedActorState,
    zigbee2mqtt::Aqara_MCCGQ12LM,
};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub enum Entity {
    AqaraMCCGQ12LM(Aqara_MCCGQ12LM::AqaraMCCGQ12LM),
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
}

pub enum Message {
    NewEvent(NewEvent),
}

pub struct DoorSensorHandler {
    shared_actor_state: SharedActorState,
}

// TODO: write the name from config too
impl DoorSensorHandler {
    pub const NAME: &str = "door-sensor";

    async fn save_values_to_db(
        &self,
        event_id: Uuid,
        friendly_name: String,
        ieee_addr: String,
        contact: bool,
        battery: i64,
    ) -> Result<(), anyhow::Error> {
        sqlx::query!(
            "INSERT INTO door_sensor (event_id, name, ieee_addr, contact, battery) VALUES ($1, $2, $3, $4, $5)",
            event_id,
            friendly_name,
            ieee_addr,
            contact,
            battery,
        ).execute(&self.shared_actor_state.db).await?;

        Ok(())
    }

    fn send_to_all_listeners(ieee_addr: String, contact: bool) -> Result<(), anyhow::Error> {
        let members = ractor::pg::get_members(&DoorEventsSupervisor::GROUP_NAME.to_owned());
        for member in members {
            let event = if contact {
                DoorEvents::Closed {
                    ieee_addr: ieee_addr.clone(),
                }
            } else {
                DoorEvents::Opened {
                    ieee_addr: ieee_addr.clone(),
                }
            };

            member.send_message(event)?;
        }

        Ok(())
    }

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::NewEvent(event) => match event.entity {
                Entity::AqaraMCCGQ12LM(aqara_mccgq12_lm) => {
                    self.save_values_to_db(
                        event.event_id,
                        aqara_mccgq12_lm.device.friendly_name,
                        aqara_mccgq12_lm.device.ieee_addr.clone(),
                        aqara_mccgq12_lm.contact,
                        aqara_mccgq12_lm.battery,
                    )
                    .await?;

                    Self::send_to_all_listeners(
                        aqara_mccgq12_lm.device.ieee_addr.clone(),
                        aqara_mccgq12_lm.contact,
                    )?
                }
            },
        }

        Ok(())
    }
}

impl Worker for DoorSensorHandler {
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

pub struct DoorSensorHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}
impl WorkerBuilder<DoorSensorHandler, ()> for DoorSensorHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (DoorSensorHandler, ()) {
        (
            DoorSensorHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
