use crate::actors::devices::handler::DeviceHandler;
use crate::repo::door::DoorReading;
use crate::{
    actors::devices::door_events::{
        DoorEvents, DoorEventsMessage, DoorEventsSupervisor, DoorEventsType,
    },
    state::AppState,
};
use uuid::Uuid;

pub enum Entity {
    Zigbee {
        address: String,
        friendly_name: String,
        contact: bool,
        battery: i64,
    },
}

pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
}

pub enum Message {
    NewEvent(NewEvent),
}

pub struct DoorSensorHandler {
    shared_actor_state: AppState,
}

// TODO: write the name from config too
// TODO: turn into derived events instead of raw (the door sensor pings)
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
        self.shared_actor_state
            .repos
            .door()
            .append_reading(&DoorReading {
                event_id,
                friendly_name,
                ieee_addr,
                contact,
                battery,
            })
            .await?;

        Ok(())
    }

    fn send_to_all_listeners(
        event_id: Uuid,
        ieee_addr: String,
        contact: bool,
    ) -> Result<(), anyhow::Error> {
        let members = ractor::pg::get_members(&DoorEventsSupervisor::GROUP_NAME.to_owned());
        for member in members {
            let event = DoorEvents {
                ieee_addr: ieee_addr.clone(),
                event_id,
                event: if contact {
                    DoorEventsType::Closed
                } else {
                    DoorEventsType::Opened
                },
            };

            member.send_message(DoorEventsMessage::Event(event))?;
        }

        Ok(())
    }

    async fn handle(&self, message: Message) -> Result<(), anyhow::Error> {
        match message {
            Message::NewEvent(event) => match event.entity {
                Entity::Zigbee {
                    address,
                    friendly_name,
                    contact,
                    battery,
                } => {
                    self.save_values_to_db(
                        event.event_id,
                        friendly_name,
                        address.clone(),
                        contact,
                        battery,
                    )
                    .await?;

                    Self::send_to_all_listeners(event.event_id, address, contact)?
                }
            },
        }

        Ok(())
    }
}

impl DeviceHandler for DoorSensorHandler {
    const NAME: &'static str = DoorSensorHandler::NAME;
    const WORKERS: usize = 1;

    type Message = Message;
    type State = ();

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    async fn handle(&self, message: Self::Message, _state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message).await
    }
}
