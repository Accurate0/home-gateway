use crate::{feature_flag::FeatureFlagClient, settings::Settings};
use http::Method;
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
    settings: Settings,
    feature_flag_client: FeatureFlagClient,
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

    #[tracing::instrument(name = "selfbot", skip(self, _wid, _factory, msg, _state))]
    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), SelfBotMessage>>,
        Job { msg, .. }: Job<(), SelfBotMessage>,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match msg {
            SelfBotMessage::SendMessage(self_bot_message_request) => {
                if self
                    .feature_flag_client
                    .is_feature_enabled("home-gateway-selfbot-killswitch", false)
                    .await
                {
                    tracing::warn!("selfbot kill switch is enabled, not sending message");
                    return Ok(());
                }

                let url = format!("{}/message", self.settings.selfbot_api_base);
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
    pub settings: Settings,
    feature_flag_client: FeatureFlagClient,
}
impl WorkerBuilder<SelfBotWorker, ()> for SelfBotWorkerBuilder {
    fn build(&mut self, _wid: usize) -> (SelfBotWorker, ()) {
        (
            SelfBotWorker {
                feature_flag_client: self.feature_flag_client.clone(),
                settings: self.settings.clone(),
                client: self.client.clone(),
            },
            (),
        )
    }
}
