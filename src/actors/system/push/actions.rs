use std::collections::HashMap;

use crate::settings::{NotifyAction, NotifyActionKind, WorkflowDefinition};

use super::types::{PushAction, PushActionKind};

pub fn resolve(
    workflows: &HashMap<String, WorkflowDefinition>,
    actions: &[NotifyAction],
) -> Vec<PushAction> {
    actions
        .iter()
        .filter_map(|action| {
            let kind = match &action.action {
                NotifyActionKind::RunWorkflow { workflow } => {
                    let Some(target) = workflows.get(workflow).map(WorkflowDefinition::body) else {
                        tracing::warn!(
                            "notify action `{}` names unknown workflow `{workflow}`",
                            action.label
                        );

                        return None;
                    };

                    PushActionKind::RunWorkflow {
                        slug: target.slug.clone(),
                    }
                }
                NotifyActionKind::Snooze { seconds } => {
                    PushActionKind::Snooze { seconds: *seconds }
                }
                NotifyActionKind::Dismiss => PushActionKind::Dismiss,
                NotifyActionKind::Acknowledge => PushActionKind::Acknowledge,
            };

            Some(PushAction {
                label: action.label.clone(),
                kind,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::resolve;
    use crate::actors::system::push::types::PushActionKind;
    use crate::settings::{NotifyAction, NotifyActionKind, WorkflowDefinition};

    fn workflows() -> HashMap<String, WorkflowDefinition> {
        let defined: Vec<WorkflowDefinition> = serde_yaml::from_str(
            r#"
- name: Close The Door
  slug: close-the-door
  group: Reminders
  on: { type: cron, schedule: "0 12 * * FRI" }
  modes: [home]
  run: []
"#,
        )
        .expect("expected the workflow to parse");

        defined
            .into_iter()
            .map(|workflow| (workflow.body().name.clone(), workflow))
            .collect()
    }

    fn action(label: &str, kind: NotifyActionKind) -> NotifyAction {
        NotifyAction {
            label: label.to_owned(),
            action: kind,
        }
    }

    #[test]
    fn the_simple_kinds_map_straight_through() {
        let resolved = resolve(
            &workflows(),
            &[
                action("Later", NotifyActionKind::Snooze { seconds: 600 }),
                action("Go away", NotifyActionKind::Dismiss),
                action("Done", NotifyActionKind::Acknowledge),
            ],
        );

        assert_eq!(resolved.len(), 3);
        assert_eq!(resolved[0].label, "Later");
        assert!(matches!(
            resolved[0].kind,
            PushActionKind::Snooze { seconds: 600 }
        ));
        assert!(matches!(resolved[1].kind, PushActionKind::Dismiss));
        assert!(matches!(resolved[2].kind, PushActionKind::Acknowledge));
    }

    #[test]
    fn a_run_workflow_action_resolves_to_its_slug() {
        let resolved = resolve(
            &workflows(),
            &[action(
                "Run",
                NotifyActionKind::RunWorkflow {
                    workflow: "Close The Door".to_owned(),
                },
            )],
        );

        assert_eq!(resolved.len(), 1);
        assert!(matches!(
            &resolved[0].kind,
            PushActionKind::RunWorkflow { slug } if slug == "close-the-door"
        ));
    }

    #[test]
    fn an_action_naming_an_unknown_workflow_is_dropped() {
        let resolved = resolve(
            &workflows(),
            &[action(
                "Run",
                NotifyActionKind::RunWorkflow {
                    workflow: "no-such-workflow".to_owned(),
                },
            )],
        );

        assert!(resolved.is_empty());
    }
}
