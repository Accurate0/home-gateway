use serde::Deserialize;
use std::collections::HashMap;

/// Named notify targets (`name -> source`) declared under `notify_targets:`.
pub type NotifyTargets = HashMap<String, NotifySource>;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotifySource {
    /// Push notification to all registered HomeGateway Android devices.
    AndroidApp,
}

/// A reference to a notify destination: either the name of a target declared
/// under `notify_targets:`, or an inline source.
#[derive(Debug, Deserialize, Clone)]
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
