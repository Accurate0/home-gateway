use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum PushNotificationActionKind {
    RunWorkflow,
    Snooze,
    Dismiss,
    Acknowledge,
}
