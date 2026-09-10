#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationInteraction {
    Acknowledged,
    Opened,
    Dismissed,
    Snoozed,
    Swiped,
}

impl NotificationInteraction {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationInteraction::Acknowledged => "acknowledged",
            NotificationInteraction::Opened => "opened",
            NotificationInteraction::Dismissed => "dismissed",
            NotificationInteraction::Snoozed => "snoozed",
            NotificationInteraction::Swiped => "swiped",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "acknowledged" => Some(NotificationInteraction::Acknowledged),
            "opened" => Some(NotificationInteraction::Opened),
            "dismissed" => Some(NotificationInteraction::Dismissed),
            "snoozed" => Some(NotificationInteraction::Snoozed),
            "swiped" => Some(NotificationInteraction::Swiped),
            _ => None,
        }
    }
}
