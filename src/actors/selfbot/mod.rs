use crate::types::SharedActorState;
use http::Method;
use open_feature::EvaluationContext;
use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use types::SelfBotMessageRequest;

pub mod spawn;
pub mod types;

pub enum SelfBotMessage {
    SendMessage(SelfBotMessageRequest),
}

pub struct SelfBotWorker {
    client: reqwest_middleware::ClientWithMiddleware,
    shared_actor_state: SharedActorState,
}

impl SelfBotWorker {
    pub const NAME: &str = "selfbot";
}

impl Worker for SelfBotWorker {
    type Key = ();
    type Message = SelfBotMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), SelfBotMessage>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), SelfBotMessage>>,
        Job { msg, .. }: Job<(), SelfBotMessage>,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match msg {
            SelfBotMessage::SendMessage(self_bot_message_request) => {
                let evaluation_context = EvaluationContext::default()
                    .with_custom_field("message", self_bot_message_request.message.clone());
                if self
                    .shared_actor_state
                    .feature_flag_client
                    .is_feature_enabled(
                        "home-gateway-selfbot-killswitch",
                        false,
                        evaluation_context,
                    )
                    .await
                {
                    tracing::warn!(
                        "selfbot kill switch is enabled, not sending message: {}",
                        self_bot_message_request.message
                    );
                    return Ok(());
                }

                let settings = self.shared_actor_state.settings.load();
                let url = format!("{}/message", settings.selfbot_api_base);
                let response = self
                    .client
                    .request(Method::POST, url)
                    .json(&self_bot_message_request)
                    .send()
                    .await;

                if let Err(e) = response {
                    tracing::error!("error sending selfbot request: {e}");
                }
            }
        }

        Ok(())
    }
}

pub struct SelfBotWorkerBuilder {
    pub client: reqwest_middleware::ClientWithMiddleware,
    pub shared_actor_state: SharedActorState,
}
impl WorkerBuilder<SelfBotWorker, ()> for SelfBotWorkerBuilder {
    fn build(&mut self, _wid: usize) -> (SelfBotWorker, ()) {
        (
            SelfBotWorker {
                client: self.client.clone(),
                shared_actor_state: self.shared_actor_state.clone(),
            },
            (),
        )
    }
}
