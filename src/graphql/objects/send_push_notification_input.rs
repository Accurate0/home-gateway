use async_graphql::InputObject;

use crate::graphql::objects::push_notification_acknowledge_input::PushNotificationAcknowledgeInput;
use crate::graphql::objects::push_notification_action_input::PushNotificationActionInput;
use crate::settings::NotifyCategory;

#[derive(InputObject)]
#[graphql(name = "SendPushNotificationInput")]
pub struct SendPushNotificationInput {
    pub title: String,
    pub body: String,
    pub category: NotifyCategory,
    pub tag: Option<String>,
    #[graphql(default)]
    pub actions: Vec<PushNotificationActionInput>,
    pub acknowledge: Option<PushNotificationAcknowledgeInput>,
}
