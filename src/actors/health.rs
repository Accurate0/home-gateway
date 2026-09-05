use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lifecycle {
    Running,
    Skipped,
    Unavailable,
}

#[derive(Clone, Default)]
pub struct ActorHealthRegistry(Arc<RwLock<BTreeMap<&'static str, Lifecycle>>>);

impl ActorHealthRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, name: &'static str, lifecycle: Lifecycle) {
        let mut actors = self.0.write().unwrap_or_else(|e| e.into_inner());
        actors.insert(name, lifecycle);
    }

    pub fn snapshot(&self) -> Vec<(&'static str, Lifecycle)> {
        let actors = self.0.read().unwrap_or_else(|e| e.into_inner());

        actors.iter().map(|(name, state)| (*name, *state)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_registry_knows_about_nothing() {
        assert!(ActorHealthRegistry::new().snapshot().is_empty());
    }

    #[test]
    fn the_latest_record_for_an_actor_wins() {
        let registry = ActorHealthRegistry::new();

        registry.record("light", Lifecycle::Running);
        registry.record("light", Lifecycle::Unavailable);
        registry.record("solar", Lifecycle::Skipped);

        assert_eq!(
            registry.snapshot(),
            vec![
                ("light", Lifecycle::Unavailable),
                ("solar", Lifecycle::Skipped)
            ]
        );
    }
}
