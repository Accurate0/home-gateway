use crate::{actors::light::record_light_state, state::SharedActorState};
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use uuid::Uuid;

pub mod spawn;

pub enum Entity {
    Zigbee {
        address: String,
        friendly_name: String,
        voltage: i64,
        power: i64,
        current: f64,
        energy: f64,
        state: Option<String>,
    },
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

    #[allow(clippy::too_many_arguments)]
    async fn save_values_to_db(
        &self,
        event_id: Uuid,
        friendly_name: &String,
        ieee_addr: &String,
        voltage: i64,
        power: i64,
        current: f64,
        energy: f64,
    ) -> Result<(), anyhow::Error> {
        sqlx::query!(
            "INSERT INTO smart_switch (event_id, name, ieee_addr, voltage, power, current_f64, energy) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            event_id,
            friendly_name,
            ieee_addr,
            voltage,
            power,
            current,
            energy
        ).execute(&self.shared_actor_state.db).await?;

        Ok(())
    }

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::NewEvent(event) => match event.entity {
                Entity::Zigbee {
                    address,
                    friendly_name,
                    voltage,
                    power,
                    current,
                    energy,
                    state,
                } => {
                    self.save_values_to_db(
                        event.event_id,
                        &friendly_name,
                        &address,
                        voltage,
                        power,
                        current,
                        energy,
                    )
                    .await?;

                    let is_light = self.shared_actor_state.devices.light(&address).is_some();

                    match (is_light, state) {
                        (true, Some(state)) => {
                            record_light_state(
                                &self.shared_actor_state,
                                event.event_id,
                                address,
                                state,
                            )
                            .await?;
                        }
                        (true, None) => {
                            tracing::debug!(
                                "smart switch {address} acts as a light but reported no state"
                            );
                        }
                        (false, _) => {}
                    }
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
