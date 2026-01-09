use crate::{notify::notify, types::SharedActorState};
use ractor::Actor;
use types::MaccasOfferIngest;

pub mod types;

pub enum MaccasMessage {
    NewOffer(MaccasOfferIngest),
}

pub struct MaccasActor {
    pub shared_actor_state: SharedActorState,
}

impl MaccasActor {
    pub const NAME: &str = "maccas";
}

impl Actor for MaccasActor {
    type Msg = MaccasMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    #[tracing::instrument(name = "maccas-actor", skip(self, _myself, message, _state))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            MaccasMessage::NewOffer(maccas_offer_ingest) => {
                let name_to_match_on = maccas_offer_ingest.details.short_name;
                let settings = self.shared_actor_state.settings.load();
                for offer_to_try_match in &settings.maccas.offers {
                    let try_match = &offer_to_try_match.match_names;
                    let matched = try_match.iter().any(|m| name_to_match_on.contains(m));
                    if matched {
                        let message = format!("{name_to_match_on} available now");
                        notify(&offer_to_try_match.notify, message, true);
                    }
                }
            }
        }
        Ok(())
    }
}
