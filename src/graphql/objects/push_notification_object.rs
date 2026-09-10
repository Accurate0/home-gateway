use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::graphql::objects::push_notification_action_object::PushNotificationActionObject;
use crate::graphql::objects::push_notification_interaction_object::PushNotificationInteractionObject;

#[derive(SimpleObject)]
#[graphql(name = "PushNotification")]
pub struct PushNotificationObject {
    pub id: Uuid,
    pub tag: String,
    pub title: String,
    pub body: String,
    pub category: String,
    pub actions: Vec<PushNotificationActionObject>,
    pub send_count: i32,
    pub next_reminder_at: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub interactions: Vec<PushNotificationInteractionObject>,
}
