use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(name = "HomePushNotificationActionKind")]
pub enum PushNotificationActionKind {
    RunWorkflow,
    Snooze,
    Dismiss,
    Acknowledge,
}
