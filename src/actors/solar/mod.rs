use crate::{routes::ingest::solar::SolarIngestPayload, types::SharedActorState};
use ractor::Actor;
use tracing::Level;

pub enum SolarMessage {
    NewData(SolarIngestPayload),
}

pub struct SolarIngestActor {
    #[allow(unused)]
    pub shared_actor_state: SharedActorState,
}

impl SolarIngestActor {
    pub const NAME: &str = "solar-ingest";
}

impl Actor for SolarIngestActor {
    type Msg = SolarMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    #[tracing::instrument(name = "solar-ingest", skip(self, _myself, message, _state), level = Level::TRACE)]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            SolarMessage::NewData(_solar_ingest_payload) => {}
        }

        Ok(())
    }
}
