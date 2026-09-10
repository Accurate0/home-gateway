use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use gcp_auth::TokenProvider;
use http::{Method, StatusCode};
use open_feature::EvaluationContext;
use ractor::{Actor, ActorProcessingErr, ActorRef};
use types::{FcmAndroidConfig, FcmMessage, FcmSendRequest, PushAction};
use uuid::Uuid;

use crate::repo::push::{NewPushNotification, PushNotificationRow};
use crate::settings::{NotifyAcknowledge, NotifyCategory};
use crate::state::AppState;

pub mod spawn;
pub mod types;

const FCM_SCOPE: &str = "https://www.googleapis.com/auth/firebase.messaging";

pub struct PushNotification {
    pub title: String,
    pub body: String,
    pub category: NotifyCategory,
    pub tag: String,
    pub actions: Vec<PushAction>,
    pub acknowledge: Option<NotifyAcknowledge>,
}

pub enum PushMessage {
    Send(PushNotification),
    ReminderDue(Uuid),
}

pub struct PushActor {
    client: reqwest_middleware::ClientWithMiddleware,
    shared_actor_state: AppState,
    token_provider: Option<Arc<dyn TokenProvider>>,
}

impl PushActor {
    pub const NAME: &str = "push";

    fn data_payload(
        notification: &PushNotification,
        notification_id: Option<Uuid>,
    ) -> HashMap<String, String> {
        let actions = match serde_json::to_string(&notification.actions) {
            Ok(json) => json,
            Err(e) => {
                tracing::error!("failed to serialise push actions: {e}");
                "[]".to_string()
            }
        };

        let mut data = HashMap::from([
            ("title".to_string(), notification.title.clone()),
            ("body".to_string(), notification.body.clone()),
            (
                "category".to_string(),
                notification.category.as_str().to_string(),
            ),
            ("tag".to_string(), notification.tag.clone()),
            ("actions".to_string(), actions),
        ]);

        if let Some(id) = notification_id {
            data.insert("notification_id".to_string(), id.to_string());
        }

        data
    }

    fn schedule_reminder(myself: &ActorRef<PushMessage>, id: Uuid, at: DateTime<Utc>) {
        let remaining = (at - Utc::now()).to_std().unwrap_or(Duration::ZERO);

        tracing::info!("reminder for notification {id} due at {at}");

        myself.send_after(remaining, move || PushMessage::ReminderDue(id));
    }

    fn from_row(row: &PushNotificationRow) -> Option<PushNotification> {
        let Some(category) = NotifyCategory::parse(&row.category) else {
            tracing::error!(
                "notification {} has unknown category '{}'",
                row.id,
                row.category
            );
            return None;
        };

        let actions = match serde_json::from_value(row.actions.clone()) {
            Ok(actions) => actions,
            Err(e) => {
                tracing::error!("notification {} has invalid actions: {e}", row.id);
                return None;
            }
        };

        Some(PushNotification {
            title: row.title.clone(),
            body: row.body.clone(),
            category,
            tag: row.tag.clone(),
            actions,
            acknowledge: None,
        })
    }

    async fn track(
        &self,
        myself: &ActorRef<PushMessage>,
        notification: &PushNotification,
        acknowledge: NotifyAcknowledge,
    ) -> Option<Uuid> {
        let actions = match serde_json::to_value(&notification.actions) {
            Ok(actions) => actions,
            Err(e) => {
                tracing::error!("failed to encode push actions for tracking: {e}");
                return None;
            }
        };

        let recorded = self
            .shared_actor_state
            .repos
            .push()
            .record_sent(NewPushNotification {
                tag: &notification.tag,
                title: &notification.title,
                body: &notification.body,
                category: notification.category.as_str(),
                actions,
                remind_after: acknowledge.remind_after,
                reminders: acknowledge.reminders,
            })
            .await;

        match recorded {
            Ok(row) => {
                if let Some(at) = row.next_reminder_at {
                    Self::schedule_reminder(myself, row.id, at);
                }

                Some(row.id)
            }
            Err(e) => {
                tracing::error!("failed to record tracked notification: {e}");
                None
            }
        }
    }

    async fn remind(&self, myself: &ActorRef<PushMessage>, id: Uuid) {
        let due = match self
            .shared_actor_state
            .repos
            .push()
            .take_due_reminder(id)
            .await
        {
            Ok(due) => due,
            Err(e) => {
                tracing::error!("failed to take reminder for notification {id}: {e}");
                return;
            }
        };

        let Some(row) = due else {
            tracing::info!("reminder for notification {id} no longer due, skipping");
            return;
        };

        if let Some(at) = row.next_reminder_at {
            Self::schedule_reminder(myself, row.id, at);
        } else {
            tracing::info!("notification {id} has used its last reminder");
        }

        let Some(notification) = Self::from_row(&row) else {
            return;
        };

        tracing::info!(
            "reminding unacknowledged notification {id} (send {})",
            row.send_count
        );

        self.deliver(&notification, Some(row.id)).await;
    }

    async fn restore_reminders(&self, myself: &ActorRef<PushMessage>) {
        let repo = self.shared_actor_state.repos.push();

        let pending = match repo.pending_reminders().await {
            Ok(pending) => pending,
            Err(e) => {
                tracing::error!("failed to load pending push reminders: {e}");
                return;
            }
        };

        let catch_up_within = self
            .shared_actor_state
            .settings
            .workflow
            .timers
            .catch_up_within;
        let now = Utc::now();

        for row in pending {
            let Some(at) = row.next_reminder_at else {
                continue;
            };

            if now - at > catch_up_within {
                tracing::warn!(
                    "dropping reminder for notification {} that was due at {at}",
                    row.id
                );

                if let Err(e) = repo.clear_reminder(row.id).await {
                    tracing::error!("failed to clear stale reminder {}: {e}", row.id);
                }

                continue;
            }

            Self::schedule_reminder(myself, row.id, at);
        }
    }

    async fn deliver(&self, notification: &PushNotification, notification_id: Option<Uuid>) {
        let Some(token_provider) = &self.token_provider else {
            tracing::warn!(
                "no fcm service account configured, skipping push: {}",
                notification.title
            );
            return;
        };

        let evaluation_context =
            EvaluationContext::default().with_custom_field("message", notification.body.clone());
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
            tracing::warn!(
                "notification kill switch is enabled, not sending: {}",
                notification.title
            );
            return;
        }

        let project_id = self.shared_actor_state.settings.fcm_project_id.clone();

        let access_token = match token_provider.token(&[FCM_SCOPE]).await {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("failed to mint fcm access token: {e}");
                return;
            }
        };

        let device_tokens: Vec<String> = match self.shared_actor_state.repos.push().tokens().await {
            Ok(tokens) => tokens,
            Err(e) => {
                tracing::error!("failed to load push tokens: {e}");
                return;
            }
        };

        if device_tokens.is_empty() {
            tracing::info!("no registered push tokens, skipping push");
            return;
        }

        let data = Self::data_payload(notification, notification_id);

        for device_token in device_tokens {
            self.send_to_token(access_token.as_str(), &project_id, device_token, &data)
                .await;
        }
    }

    async fn send_to_token(
        &self,
        access_token: &str,
        project_id: &str,
        device_token: String,
        data: &HashMap<String, String>,
    ) {
        let url = format!("https://fcm.googleapis.com/v1/projects/{project_id}/messages:send");
        let payload = FcmSendRequest {
            message: FcmMessage {
                token: device_token.clone(),
                data: data.clone(),
                android: FcmAndroidConfig { priority: "high" },
            },
        };

        let response = self
            .client
            .request(Method::POST, url)
            .with_extension(crate::http::UrlTemplate(
                "/v1/projects/{project_id}/messages:send",
            ))
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                if status == StatusCode::NOT_FOUND {
                    tracing::info!("pruning unregistered push token");
                    if let Err(e) = self
                        .shared_actor_state
                        .repos
                        .push()
                        .delete_token(&device_token)
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

impl Actor for PushActor {
    type Msg = PushMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn post_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        self.restore_reminders(&myself).await;

        Ok(())
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PushMessage::Send(notification) => {
                let notification_id = match notification.acknowledge {
                    Some(acknowledge) => self.track(&myself, &notification, acknowledge).await,
                    None => None,
                };

                self.deliver(&notification, notification_id).await;
            }
            PushMessage::ReminderDue(id) => self.remind(&myself, id).await,
        }

        Ok(())
    }
}
