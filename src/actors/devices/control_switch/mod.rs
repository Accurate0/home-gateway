use crate::actors::devices::handler::DeviceHandler;
use crate::{event_bus::EventBusMessage, state::AppState};
use uuid::Uuid;

#[derive(Debug)]
pub enum Entity {
    Zigbee { address: String, action: String },
}

#[derive(Debug)]
pub struct NewEvent {
    pub event_id: Uuid,
    pub entity: Entity,
}

pub enum ControlSwitchMessage {
    NewEvent(NewEvent),
}

pub struct ControlSwitchHandler {
    shared_actor_state: AppState,
}

impl ControlSwitchHandler {
    pub const NAME: &str = "control-switch";

    async fn handle(&self, message: ControlSwitchMessage) -> Result<(), anyhow::Error> {
        match message {
            ControlSwitchMessage::NewEvent(event) => match &event.entity {
                Entity::Zigbee { address, action } => {
                    self.shared_actor_state
                        .event_bus
                        .publish(EventBusMessage::SwitchAction {
                            event_id: event.event_id,
                            ieee_addr: address.clone(),
                            action: action.clone(),
                        });
                }
            },
        }

        Ok(())
    }
}

impl DeviceHandler for ControlSwitchHandler {
    const NAME: &'static str = ControlSwitchHandler::NAME;
    const WORKERS: usize = 3;

    type Message = ControlSwitchMessage;
    type State = ();

    fn new(shared_actor_state: AppState) -> Self {
        Self { shared_actor_state }
    }

    async fn handle(&self, message: Self::Message, _state: &mut Self::State) -> anyhow::Result<()> {
        Self::handle(self, message).await
    }
}
