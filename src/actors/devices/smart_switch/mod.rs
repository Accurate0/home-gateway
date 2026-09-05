use crate::actors::devices::handler::DeviceHandler;
use crate::repo::smart_switch::SmartSwitchReading;
use crate::{actors::devices::light::record_light_state, state::AppState};
use uuid::Uuid;

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
    shared_actor_state: AppState,
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
        self.shared_actor_state
            .repos
            .smart_switch()
            .append(&SmartSwitchReading {
                event_id,
                friendly_name: friendly_name.to_owned(),
                ieee_addr: ieee_addr.to_owned(),
                voltage,
                power,
                current,
                energy,
            })
            .await?;

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

impl DeviceHandler for SmartSwitchHandler {
    const NAME: &'static str = SmartSwitchHandler::NAME;
    const WORKERS: usize = 3;

    type Message = Message;
    type State = ();

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    async fn handle(&self, message: Self::Message, _state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message).await
    }
}
