use std::sync::Arc;

use gcp_auth::TokenProvider;
use http::{Method, StatusCode};
use open_feature::EvaluationContext;
use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, Worker, WorkerBuilder, WorkerId},
};
use types::{FcmMessage, FcmNotification, FcmSendRequest};

use crate::types::SharedActorState;

pub mod spawn;
pub mod types;

/// OAuth2 scope required to call the FCM HTTP v1 API.
const FCM_SCOPE: &str = "https://www.googleapis.com/auth/firebase.messaging";

pub enum PushMessage {
    Send { title: String, body: String },
}

pub struct PushWorker {
    client: reqwest_middleware::ClientWithMiddleware,
    shared_actor_state: SharedActorState,
    /// `None` when no FCM service account is configured; sends are then skipped.
    token_provider: Option<Arc<dyn TokenProvider>>,
}

impl PushWorker {
    pub const NAME: &str = "push";

    async fn send_to_token(
        &self,
        access_token: &str,
        project_id: &str,
        device_token: String,
        title: &str,
        body: &str,
    ) {
        let url = format!("https://fcm.googleapis.com/v1/projects/{project_id}/messages:send");
        let payload = FcmSendRequest {
            message: FcmMessage {
                token: device_token.clone(),
                notification: FcmNotification {
                    title: title.to_string(),
                    body: body.to_string(),
                },
            },
        };

        let response = self
            .client
            .request(Method::POST, url)
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                // FCM returns 404 NOT_FOUND for tokens that are no longer valid
                // (app uninstalled / token rotated). Prune them so we stop trying.
                if status == StatusCode::NOT_FOUND {
                    tracing::info!("pruning unregistered push token");
                    if let Err(e) = sqlx::query("DELETE FROM android_push_tokens WHERE token = $1")
                        .bind(&device_token)
                        .execute(&self.shared_actor_state.db)
                        .await
                    {
                        tracing::error!("failed to prune push token: {e}");
                    }
                } else if !status.is_success() {
                    let text = resp.text().await.unwrap_or_default();
                    tracing::error!("fcm send failed ({status}): {text}");
                }
            }
            Err(e) => tracing::error!("error sending fcm request: {e}"),
        }
    }
}

impl Worker for PushWorker {
    type Key = ();
    type Message = PushMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), PushMessage>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), PushMessage>>,
        Job { msg, .. }: Job<(), PushMessage>,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match msg {
            PushMessage::Send { title, body } => {
                let Some(token_provider) = &self.token_provider else {
                    tracing::warn!("no fcm service account configured, skipping push: {title}");
                    return Ok(());
                };

                let evaluation_context =
                    EvaluationContext::default().with_custom_field("message", body.clone());
                if self
                    .shared_actor_state
                    .feature_flag_client
                    .is_feature_enabled(
                        "home-gateway-notification-killswitch",
                        false,
                        evaluation_context,
                    )
                    .await
                {
                    tracing::warn!("notification kill switch is enabled, not sending: {title}");
                    return Ok(());
                }

                let settings = self.shared_actor_state.settings.load();
                let project_id = settings.fcm_project_id.clone();
                drop(settings);

                let access_token = match token_provider.token(&[FCM_SCOPE]).await {
                    Ok(t) => t,
                    Err(e) => {
                        tracing::error!("failed to mint fcm access token: {e}");
                        return Ok(());
                    }
                };

                let device_tokens: Vec<String> =
                    match sqlx::query_scalar!("SELECT token FROM android_push_tokens")
                        .fetch_all(&self.shared_actor_state.db)
                        .await
                    {
                        Ok(tokens) => tokens,
                        Err(e) => {
                            tracing::error!("failed to load push tokens: {e}");
                            return Ok(());
                        }
                    };

                if device_tokens.is_empty() {
                    tracing::info!("no registered push tokens, skipping push");
                    return Ok(());
                }

                for device_token in device_tokens {
                    self.send_to_token(
                        access_token.as_str(),
                        &project_id,
                        device_token,
                        &title,
                        &body,
                    )
                    .await;
                }
            }
        }

        Ok(())
    }
}

pub struct PushWorkerBuilder {
    pub client: reqwest_middleware::ClientWithMiddleware,
    pub shared_actor_state: SharedActorState,
    pub token_provider: Option<Arc<dyn TokenProvider>>,
}

impl WorkerBuilder<PushWorker, ()> for PushWorkerBuilder {
    fn build(&mut self, _wid: usize) -> (PushWorker, ()) {
        (
            PushWorker {
                client: self.client.clone(),
                shared_actor_state: self.shared_actor_state.clone(),
                token_provider: self.token_provider.clone(),
            },
            (),
        )
    }
}
