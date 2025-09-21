use crate::types::SharedActorState;
use ractor::Actor;

#[allow(unused)]
pub enum VacuumMessage {
    Start,
    Stop,
    Pause,
    Home,
}

impl VacuumMessage {
    fn to_basic_operation(&self) -> &'static str {
        match self {
            VacuumMessage::Start => "start",
            VacuumMessage::Stop => "stop",
            VacuumMessage::Pause => "pause",
            VacuumMessage::Home => "return_to_base",
        }
    }
}

pub struct VacuumActor {
    pub shared_actor_state: SharedActorState,
}

impl VacuumActor {
    pub const NAME: &str = "vacuum";
    pub const VACUUM_MQTT_PREFIX: &str = "valetudo/rockrobo";
}

impl VacuumActor {
    async fn send_mqtt_state(&self, state: &str) -> Result<(), anyhow::Error> {
        let topic = format!("{}/command", Self::VACUUM_MQTT_PREFIX);
        self.shared_actor_state
            .mqtt
            .send_event_raw(topic, state)
            .await?;

        Ok(())
    }
}

impl Actor for VacuumActor {
    type Msg = VacuumMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    #[tracing::instrument(name = "vacuum-actor", skip(self, _myself, message, _state))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        self.send_mqtt_state(message.to_basic_operation()).await?;
        Ok(())
    }
}
