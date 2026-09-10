use axum::Json;
use http::StatusCode;

use serde::Deserialize;

use crate::actors::system::push::types::{PushAction, PushActionKind};
use crate::actors::system::push::{self, PushActor, PushNotification};
use crate::actors::system::rpc;
use crate::auth::{
    Auth,
    scope::{Action, Resource, Scope},
};
use crate::settings::{NotifyAcknowledge, NotifyCategory};

#[derive(Deserialize)]
pub struct PushNotifyPayload {
    #[serde(default = "default_title")]
    pub title: String,
    pub body: String,
    #[serde(default = "default_category")]
    pub category: NotifyCategory,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub actions: Vec<PushActionPayload>,
    #[serde(default)]
    pub acknowledge: Option<NotifyAcknowledge>,
}

#[derive(Deserialize)]
pub struct PushActionPayload {
    pub label: String,
    #[serde(flatten)]
    pub action: PushActionKindPayload,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PushActionKindPayload {
    RunWorkflow { slug: String },
    Snooze { seconds: u64 },
    Dismiss,
    Acknowledge,
}

fn default_title() -> String {
    "Home Gateway".to_string()
}

fn default_category() -> NotifyCategory {
    NotifyCategory::General
}

pub async fn notify(Auth(auth): Auth, Json(payload): Json<PushNotifyPayload>) -> StatusCode {
    if auth
        .require(&Scope::new(Resource::Push, Action::Write))
        .is_err()
    {
        return StatusCode::FORBIDDEN;
    }

    let actions = payload
        .actions
        .into_iter()
        .map(|action| PushAction {
            label: action.label,
            kind: match action.action {
                PushActionKindPayload::RunWorkflow { slug } => PushActionKind::RunWorkflow { slug },
                PushActionKindPayload::Snooze { seconds } => PushActionKind::Snooze { seconds },
                PushActionKindPayload::Dismiss => PushActionKind::Dismiss,
                PushActionKindPayload::Acknowledge => PushActionKind::Acknowledge,
            },
        })
        .collect();

    let tag = payload
        .tag
        .unwrap_or_else(|| format!("push:{}", payload.title));

    let message = push::PushMessage::Send(PushNotification {
        title: payload.title,
        body: payload.body,
        category: payload.category,
        tag,
        actions,
        acknowledge: payload.acknowledge,
    });

    if let Err(e) = rpc::cast(PushActor::NAME, message) {
        tracing::error!("error sending to push worker: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::NO_CONTENT
}
