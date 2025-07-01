use crate::{types::SharedActorState, zigbee2mqtt::TS011F_plug_1};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub enum Entity {
    TS011FSmartSwitch(TS011F_plug_1::Ts011fPlug1),
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
}

pub enum Message {
    NewEvent(NewEvent),
}

pub struct SmartSwitchHandler {
    shared_actor_state: SharedActorState,
}

impl SmartSwitchHandler {
    pub const NAME: &str = "smart-switch";

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::NewEvent(event) => match event.entity {
                Entity::TS011FSmartSwitch(ts011f_plug1) => {
                    sqlx::query!(
                            "INSERT INTO smart_switch (event_id, name, ieee_addr, voltage, power, current, energy) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                            event.event_id,
                            ts011f_plug1.device.friendly_name,
                            ts011f_plug1.device.ieee_addr,
                            ts011f_plug1.voltage,
                            ts011f_plug1.power,
                            ts011f_plug1.current,
                            ts011f_plug1.energy
                        ).execute(&self.shared_actor_state.db).await?;
                }
            },
        }

        Ok(())
    }
}

impl Worker for SmartSwitchHandler {
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

pub struct SmartSwitchHandlerBuilder {
    pub shared_actor_state: SharedActorState,
}
impl WorkerBuilder<SmartSwitchHandler, ()> for SmartSwitchHandlerBuilder {
    fn build(&mut self, _wid: usize) -> (SmartSwitchHandler, ()) {
        (
            SmartSwitchHandler {
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
