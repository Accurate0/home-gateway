use schemars::JsonSchema;
use serde::Deserialize;
use strum::Display;

use super::NotificationImportance;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, JsonSchema, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UnraidImportance {
    Alert,
    Warning,
    Info,
}

impl From<NotificationImportance> for UnraidImportance {
    fn from(importance: NotificationImportance) -> Self {
        match importance {
            NotificationImportance::Alert => Self::Alert,
            NotificationImportance::Warning => Self::Warning,
            NotificationImportance::Info => Self::Info,
        }
    }
}
