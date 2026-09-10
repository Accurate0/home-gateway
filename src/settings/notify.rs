use chrono::TimeDelta;
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;

/// Named notify targets (`name -> source`) declared under `notify_targets:`.
pub type NotifyTargets = HashMap<String, NotifySource>;

#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotifySource {
    /// Push notification to all registered HomeGateway Android devices.
    AndroidApp,
}

/// A reference to a notify destination: either the name of a target declared
/// under `notify_targets:`, or an inline source.
#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum NotifyRef {
    Named(String),
    Inline(NotifySource),
}

impl NotifyRef {
    fn resolve(self, targets: &NotifyTargets) -> Result<NotifySource, String> {
        match self {
            NotifyRef::Inline(source) => Ok(source),
            NotifyRef::Named(name) => targets
                .get(&name)
                .cloned()
                .ok_or_else(|| format!("unknown notify target: {name}")),
        }
    }
}

pub(crate) fn resolve_notify(
    refs: Vec<NotifyRef>,
    targets: &NotifyTargets,
) -> Result<Vec<NotifySource>, String> {
    refs.into_iter().map(|r| r.resolve(targets)).collect()
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub struct NotifyAction {
    pub label: String,
    pub action: NotifyActionKind,
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotifyActionKind {
    RunWorkflow { workflow: String },
    Snooze { seconds: u64 },
    Dismiss,
    Acknowledge,
}

pub fn validate_acknowledge(has_block: bool, has_action: bool) -> Result<(), String> {
    match (has_block, has_action) {
        (true, false) => {
            Err("notification has an `acknowledge` block but no acknowledge action".to_owned())
        }
        (false, true) => {
            Err("notification has an acknowledge action but no `acknowledge` block".to_owned())
        }
        (true, true) | (false, false) => Ok(()),
    }
}

#[derive(Debug, Deserialize, Clone, Copy, JsonSchema)]
pub struct NotifyAcknowledge {
    #[serde(deserialize_with = "crate::timedelta_format::time_delta_from_str::deserialize")]
    #[schemars(with = "String")]
    pub remind_after: TimeDelta,
    pub reminders: u32,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, JsonSchema, async_graphql::Enum)]
#[serde(rename_all = "snake_case")]
#[graphql(name = "PushNotificationCategory")]
pub enum NotifyCategory {
    Alarm,
    Door,
    Watchdog,
    General,
}

impl NotifyCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            NotifyCategory::Alarm => "alarm",
            NotifyCategory::Door => "door",
            NotifyCategory::Watchdog => "watchdog",
            NotifyCategory::General => "general",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "alarm" => Some(NotifyCategory::Alarm),
            "door" => Some(NotifyCategory::Door),
            "watchdog" => Some(NotifyCategory::Watchdog),
            "general" => Some(NotifyCategory::General),
            _ => None,
        }
    }
}
