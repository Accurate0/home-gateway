use crate::integrations::unraid::schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, cynic::Enum)]
pub enum NotificationImportance {
    Alert,
    Info,
    Warning,
}
