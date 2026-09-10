use async_graphql::SimpleObject;

use crate::actors::system::push::types::{PushAction, PushActionKind};
use crate::graphql::objects::push_notification_action_kind::PushNotificationActionKind;

#[derive(SimpleObject)]
#[graphql(name = "PushNotificationAction")]
pub struct PushNotificationActionObject {
    pub label: String,
    pub kind: PushNotificationActionKind,
    pub workflow_slug: Option<String>,
    pub snooze_seconds: Option<i64>,
}

impl From<PushAction> for PushNotificationActionObject {
    fn from(action: PushAction) -> Self {
        let (kind, workflow_slug, snooze_seconds) = match action.kind {
            PushActionKind::RunWorkflow { slug } => {
                (PushNotificationActionKind::RunWorkflow, Some(slug), None)
            }
            PushActionKind::Snooze { seconds } => (
                PushNotificationActionKind::Snooze,
                None,
                Some(i64::try_from(seconds).unwrap_or(i64::MAX)),
            ),
            PushActionKind::Dismiss => (PushNotificationActionKind::Dismiss, None, None),
            PushActionKind::Acknowledge => (PushNotificationActionKind::Acknowledge, None, None),
        };

        Self {
            label: action.label,
            kind,
            workflow_slug,
            snooze_seconds,
        }
    }
}
