use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

use crate::graphql::objects::notification_interaction_kind::NotificationInteractionKind;

#[derive(SimpleObject)]
#[graphql(name = "PushNotificationInteraction")]
pub struct PushNotificationInteractionObject {
    pub kind: NotificationInteractionKind,
    pub at: DateTime<Utc>,
}
