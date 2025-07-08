use crate::settings::Settings;
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
    client: reqwest::Client,
    settings: Settings,
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
    pub client: reqwest::Client,
    pub settings: Settings,
}
impl WorkerBuilder<SelfBotWorker, ()> for SelfBotWorkerBuilder {
    fn build(&mut self, _wid: usize) -> (SelfBotWorker, ()) {
        (
            SelfBotWorker {
                settings: self.settings.clone(),
                client: self.client.clone(),
            },
            (),
        )
    }
}
