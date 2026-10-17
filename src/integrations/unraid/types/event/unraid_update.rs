use super::{UnraidChange, UnraidMetric};
use crate::integrations::unraid::types::UnraidImportance;

#[derive(Debug, Clone, PartialEq)]
pub enum UnraidUpdate {
    Changed {
        change: UnraidChange,
        state: String,
        previous: String,
    },
    Reading {
        metric: UnraidMetric,
        value: f64,
    },
    Notification {
        importance: UnraidImportance,
        title: String,
        description: String,
        link: Option<String>,
    },
}
