pub mod context;
pub mod cron_task;
pub mod cron_tasks;
pub mod error;
pub mod queries;
pub mod runner;
pub mod task;
pub mod tasks;

pub use context::AdhocTaskContext;
pub use cron_task::AdhocCronTask;
pub use error::AdhocTaskError;
pub use task::AdhocTask;

pub fn registry() -> Vec<&'static dyn AdhocTask> {
    let mut tasks = tasks::all();
    tasks.sort_by_key(|task| task.ordinal());

    tasks
}

pub fn cron_registry() -> Vec<&'static dyn AdhocCronTask> {
    let mut tasks = cron_tasks::all();
    tasks.sort_by_key(|task| task.name());

    tasks
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    fn manifest(tasks: &[&'static dyn AdhocTask]) -> String {
        tasks
            .iter()
            .map(|task| {
                format!(
                    "{}  {}  {}  {}",
                    task.ordinal(),
                    task.name(),
                    task.flag().unwrap_or("-"),
                    task::checksum(task.source()),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn cron_manifest(tasks: &[&'static dyn AdhocCronTask]) -> String {
        tasks
            .iter()
            .map(|task| {
                format!(
                    "{}  {}  {}",
                    task.name(),
                    task.schedule().expression(),
                    task.flag().unwrap_or("-"),
                )
            })
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
    fn cron_schedules_have_a_next_occurrence() {
        for task in cron_registry() {
            assert!(
                task.schedule().time_until_next().is_ok(),
                "{} has no next occurrence",
                task.name()
            );
        }
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
            .map(|task| task.name())
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
            .map(|task| task.ordinal())
            .collect::<Vec<_>>();

        let mut sorted = ordinals.clone();
        sorted.sort_unstable();
        sorted.dedup();

        assert_eq!(ordinals, sorted);
    }

    #[test]
    fn names_are_unique() {
        let tasks = registry();
        let unique = tasks.iter().map(|task| task.name()).collect::<HashSet<_>>();

        assert_eq!(tasks.len(), unique.len());
    }

    #[test]
    fn flags_are_unique() {
        let flags = registry()
            .iter()
            .filter_map(|task| task.flag())
            .collect::<Vec<_>>();

        let unique = flags.iter().collect::<HashSet<_>>();

        assert_eq!(flags.len(), unique.len());
    }

    #[test]
    fn checksums_are_populated() {
        for task in registry() {
            assert!(
                !task.source().is_empty(),
                "{} has empty source",
                task.name()
            );
        }
    }
}
