pub mod subscriber;

use ractor::Actor;
use tracing::Level;

use crate::{
    event_bus::{FeatureFlagState, Recipient, Subscription},
    state::AppState,
    tracing_flag,
};

use subscriber::SamplingSubscriber;

pub enum SamplingMessage {
    Reevaluate(FeatureFlagState),
}

#[derive(Default)]
pub struct SamplingActorState {
    _subscription: Subscription,
}

pub struct SamplingActor {
    pub shared_actor_state: AppState,
}

impl SamplingActor {
    pub const NAME: &str = "sampling";
}

impl Actor for SamplingActor {
    type Msg = SamplingMessage;
    type State = SamplingActorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let subscription = self.shared_actor_state.event_bus.register(
            Self::NAME,
            Recipient::Actor(myself),
            SamplingSubscriber,
        );

        Ok(SamplingActorState {
            _subscription: subscription,
        })
    }

    #[tracing::instrument(name = "sampling-actor", skip(self, _myself, message, _state), level = Level::DEBUG)]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            SamplingMessage::Reevaluate(state) => {
                tracing::debug!("re-evaluating trace sampling after a {} flag", state.as_str());

                let ratios = tracing_flag::evaluate(&self.shared_actor_state.feature_flag_client)
                    .await;

                self.shared_actor_state.sampling.replace(ratios);
            }
        }

        Ok(())
    }
}
