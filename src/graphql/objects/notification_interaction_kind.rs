use async_graphql::Enum;

use crate::repo::notification_interaction::NotificationInteraction;

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum NotificationInteractionKind {
    Acknowledged,
    Opened,
    Dismissed,
    Snoozed,
    Swiped,
}

impl From<NotificationInteractionKind> for NotificationInteraction {
    fn from(kind: NotificationInteractionKind) -> Self {
        match kind {
            NotificationInteractionKind::Acknowledged => NotificationInteraction::Acknowledged,
            NotificationInteractionKind::Opened => NotificationInteraction::Opened,
            NotificationInteractionKind::Dismissed => NotificationInteraction::Dismissed,
            NotificationInteractionKind::Snoozed => NotificationInteraction::Snoozed,
            NotificationInteractionKind::Swiped => NotificationInteraction::Swiped,
        }
    }
}

impl From<NotificationInteraction> for NotificationInteractionKind {
    fn from(kind: NotificationInteraction) -> Self {
        match kind {
            NotificationInteraction::Acknowledged => NotificationInteractionKind::Acknowledged,
            NotificationInteraction::Opened => NotificationInteractionKind::Opened,
            NotificationInteraction::Dismissed => NotificationInteractionKind::Dismissed,
            NotificationInteraction::Snoozed => NotificationInteractionKind::Snoozed,
            NotificationInteraction::Swiped => NotificationInteractionKind::Swiped,
        }
    }
}
