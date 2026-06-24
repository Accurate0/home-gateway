#![allow(dead_code)]

use std::collections::HashMap;

use super::MAX_DEPTH;
use crate::settings::workflow::{Step, Workflow};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedAction {
    pub depth: u8,
    pub kind: &'static str,
    pub detail: String,
    pub guards: Vec<String>,
}

pub fn plan(workflows: &HashMap<String, Workflow>, steps: &[Step]) -> Vec<PlannedAction> {
    let mut out = Vec::new();
    plan_steps(workflows, steps, 0, &[], &mut out);
    out
}

fn plan_steps(
    workflows: &HashMap<String, Workflow>,
    steps: &[Step],
    depth: u8,
    guards: &[String],
    out: &mut Vec<PlannedAction>,
) {
    for step in steps {
        let mut guards = guards.to_vec();
        if let Some(when) = step.guard() {
            guards.push(when.describe());
        }

        match step {
            Step::Scene { run, .. } => plan_steps(workflows, run, depth, &guards, out),
            Step::RunWorkflow { workflow, .. } => {
                if depth >= MAX_DEPTH {
                    out.push(marker("depth_exceeded", workflow, depth, &guards));
                    continue;
                }
                match workflows.get(workflow) {
                    None => out.push(marker("unknown_workflow", workflow, depth, &guards)),
                    Some(wf) if !wf.enabled => {
                        out.push(marker("disabled", workflow, depth, &guards))
                    }
                    Some(wf) => plan_steps(workflows, &wf.run, depth + 1, &guards, out),
                }
            }
            leaf => out.push(PlannedAction {
                depth,
                kind: leaf.kind(),
                detail: leaf.describe_action().unwrap_or_default(),
                guards,
            }),
        }
    }
}

fn marker(kind: &'static str, name: &str, depth: u8, guards: &[String]) -> PlannedAction {
    PlannedAction {
        depth,
        kind,
        detail: name.to_string(),
        guards: guards.to_vec(),
    }
}

pub fn render(actions: &[PlannedAction]) -> String {
    actions
        .iter()
        .map(|a| {
            let indent = "  ".repeat(a.depth as usize);
            let guard = if a.guards.is_empty() {
                String::new()
            } else {
                format!(" [when: {}]", a.guards.join(" && "))
            };
            format!("{indent}{}: {}{guard}", a.kind, a.detail)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workflow(yaml: &str) -> Workflow {
        serde_yaml::from_str(yaml).expect("workflow yaml")
    }

    fn workflows(yaml: &str) -> HashMap<String, Workflow> {
        serde_yaml::from_str(yaml).expect("workflows yaml")
    }

    fn rendered(workflows: &HashMap<String, Workflow>, wf: &Workflow) -> String {
        render(&plan(workflows, &wf.run))
    }

    #[test]
    fn simple_light() {
        let wf = workflow(
            r#"
            run:
              - type: light
                device: "0x1"
                state: "ON"
              - type: notify
                notify: { type: android_app }
                message: "kitchen on"
            "#,
        );
        insta::assert_snapshot!(rendered(&HashMap::new(), &wf));
    }

    #[test]
    fn guarded_scene() {
        let wf = workflow(
            r#"
            run:
              - type: scene
                when: { type: presence, sensor: hallway, present: true }
                run:
                  - type: light
                    device: "0x1"
                    state: "ON"
                  - type: delay
                    seconds: 5
                  - type: light
                    device: "0x1"
                    state: "OFF"
                    when: { type: door, device: "0x2", open: false }
            "#,
        );
        insta::assert_snapshot!(rendered(&HashMap::new(), &wf));
    }

    #[test]
    fn nested_run_workflow() {
        let all = workflows(
            r#"
            entry:
              run:
                - type: light
                  device: "0x1"
                  state: "ON"
                - type: run_workflow
                  workflow: leaf
                  when: { type: time_of_day, after: "22:00:00" }
            leaf:
              run:
                - type: notify
                  notify: { type: android_app }
                  message: "from leaf"
            "#,
        );
        let entry = all.get("entry").unwrap().clone();
        insta::assert_snapshot!(rendered(&all, &entry));
    }

    #[test]
    fn recursion_depth_cap() {
        let all = workflows(
            r#"
            loop:
              run:
                - type: run_workflow
                  workflow: loop
            "#,
        );
        let entry = all.get("loop").unwrap().clone();
        insta::assert_snapshot!(rendered(&all, &entry));
    }

    #[rstest::rstest]
    fn real_workflows(
        #[files("config/workflows/*.yaml")]
        #[exclude("index")]
        path: std::path::PathBuf,
    ) {
        let yaml = std::fs::read_to_string(&path).expect("read workflow file");
        let file_workflows: Vec<Workflow> =
            serde_yaml::from_str(&yaml).expect("deserialize workflows");

        let workflows = HashMap::new();
        let mut out = String::new();
        for wf in &file_workflows {
            match wf.on() {
                Some(on) => out.push_str(&format!("# {}  (on: {})\n", wf.name, on.describe())),
                None => out.push_str(&format!("# {}  (reusable)\n", wf.name)),
            }
            if let Some(when) = wf.when() {
                out.push_str(&format!("  when: {}\n", when.describe()));
            }
            let rendered = render(&plan(&workflows, &wf.run));
            if rendered.is_empty() {
                out.push_str("  (no actions)\n");
            } else {
                for line in rendered.lines() {
                    out.push_str(&format!("  {line}\n"));
                }
            }
            out.push('\n');
        }

        let stem = path.file_stem().unwrap().to_string_lossy();
        insta::assert_snapshot!(stem.as_ref(), out);
    }

    #[test]
    fn unknown_and_disabled_refs() {
        let all = workflows(
            r#"
            off_wf:
              enabled: false
              run:
                - type: light
                  device: "0x9"
                  state: "ON"
            "#,
        );
        let entry = workflow(
            r#"
            run:
              - type: run_workflow
                workflow: missing
              - type: run_workflow
                workflow: off_wf
            "#,
        );
        insta::assert_snapshot!(rendered(&all, &entry));
    }
}
