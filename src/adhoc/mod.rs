pub mod any_cron_task;
pub mod context;
pub mod cron_task;
pub mod cron_tasks;
pub mod error;
pub mod runner;
pub mod seal;
pub mod task;
pub mod tasks;

pub use any_cron_task::AnyAdhocCronTask;
pub use context::AdhocTaskContext;
pub use cron_task::AdhocCronTask;
pub use error::AdhocTaskError;
pub use seal::{Seal, SealedTask};
pub use task::AdhocTask;

pub fn registry() -> Vec<SealedTask> {
    let mut tasks = tasks::all();
    tasks.sort_by_key(|sealed| sealed.task.ordinal());

    tasks
}

pub fn cron_registry() -> Vec<&'static dyn AnyAdhocCronTask> {
    let mut tasks = cron_tasks::all();
    tasks.sort_by_key(|task| task.name());

    tasks
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    fn manifest(tasks: &[SealedTask]) -> String {
        tasks
            .iter()
            .map(|SealedTask { task, seal }| {
                format!(
                    "{}  {}  {}  {}  {}",
                    task.ordinal(),
                    task.name(),
                    task.flag().unwrap_or("-"),
                    seal,
                    task::checksum(task.source()),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn cron_manifest(tasks: &[&'static dyn AnyAdhocCronTask]) -> String {
        tasks
            .iter()
            .map(|task| format!("{}  {}", task.name(), task.flag().unwrap_or("-")))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn registry_is_immutable() {
        insta::assert_snapshot!(manifest(&registry()));
    }

    #[test]
    fn cron_registry_is_reviewed() {
        insta::assert_snapshot!(cron_manifest(&cron_registry()));
    }

    #[test]
    fn cron_names_are_unique() {
        let tasks = cron_registry();
        let unique = tasks.iter().map(|task| task.name()).collect::<HashSet<_>>();

        assert_eq!(tasks.len(), unique.len());
    }

    #[test]
    fn cron_names_do_not_collide_with_one_shot_names() {
        let one_shot = registry()
            .iter()
            .map(|sealed| sealed.task.name())
            .collect::<HashSet<_>>();

        for task in cron_registry() {
            assert!(
                !one_shot.contains(task.name()),
                "{} is used by both registries",
                task.name()
            );
        }
    }

    #[test]
    fn ordinals_are_strictly_increasing() {
        let ordinals = registry()
            .iter()
            .map(|sealed| sealed.task.ordinal())
            .collect::<Vec<_>>();

        let mut sorted = ordinals.clone();
        sorted.sort_unstable();
        sorted.dedup();

        assert_eq!(ordinals, sorted);
    }

    #[test]
    fn names_are_unique() {
        let tasks = registry();
        let unique = tasks
            .iter()
            .map(|sealed| sealed.task.name())
            .collect::<HashSet<_>>();

        assert_eq!(tasks.len(), unique.len());
    }

    #[test]
    fn flags_are_unique() {
        let flags = registry()
            .iter()
            .filter_map(|sealed| sealed.task.flag())
            .collect::<Vec<_>>();

        let unique = flags.iter().collect::<HashSet<_>>();

        assert_eq!(flags.len(), unique.len());
    }

    #[test]
    fn checksums_are_populated() {
        for SealedTask { task, .. } in registry() {
            assert!(
                !task.source().is_empty(),
                "{} has empty source",
                task.name()
            );
        }
    }
}
