use super::{DiskTemperature, UnraidNotification};

#[derive(Debug, Clone, PartialEq)]
pub enum UnraidPush {
    Subscribed,
    Temperatures(Vec<DiskTemperature>),
    Notification(UnraidNotification),
}
