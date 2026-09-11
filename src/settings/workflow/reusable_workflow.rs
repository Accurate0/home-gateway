use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;

use super::{ContextSource, Step};
use crate::device_registry::DeviceRegistry;
use crate::settings::{DeviceAliases, NotifyActionKind, validate_acknowledge, yes};
use crate::variables::VarType;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReusableWorkflow {
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub context: Vec<ContextSource>,
    #[serde(default)]
    pub inputs: Option<BTreeMap<String, VarType>>,
    pub run: Vec<Step>,
}

impl ReusableWorkflow {
    pub fn steps(&self) -> Vec<&Step> {
        fn collect<'a>(steps: &'a [Step], out: &mut Vec<&'a Step>) {
            for step in steps {
                out.push(step);

                if let Step::Scene { run, .. } = step {
                    collect(run, out);
                }
            }
        }

        let mut out = Vec::new();
        collect(&self.run, &mut out);
        out
    }

    pub fn references_namespace(&self, namespace: &str) -> bool {
        self.steps().into_iter().any(|step| {
            let guard = step
                .guard()
                .is_some_and(|when| when.references_namespace(namespace));

            guard
                || step
                    .templates()
                    .into_iter()
                    .any(|(_, template)| template.references_namespace(namespace))
        })
    }

    pub fn run_workflow_targets(&self) -> Vec<&str> {
        self.steps()
            .into_iter()
            .filter_map(|step| match step {
                Step::RunWorkflow { workflow, .. } => Some(workflow.as_str()),
                _ => None,
            })
            .collect()
    }

    pub fn notify_action_targets(&self) -> Vec<&str> {
        self.steps()
            .into_iter()
            .filter_map(|step| match step {
                Step::Notify { actions, .. } => Some(actions),
                _ => None,
            })
            .flatten()
            .filter_map(|action| match &action.action {
                NotifyActionKind::RunWorkflow { workflow } => Some(workflow.as_str()),
                _ => None,
            })
            .collect()
    }

    pub fn validate_acknowledgements(&self) -> Result<(), String> {
        fn check(steps: &[Step]) -> Result<(), String> {
            for step in steps {
                match step {
                    Step::Notify {
                        actions,
                        acknowledge,
                        ..
                    } => {
                        let has_action = actions
                            .iter()
                            .any(|a| matches!(a.action, NotifyActionKind::Acknowledge));

                        validate_acknowledge(acknowledge.is_some(), has_action)?;
                    }
                    Step::Scene { run, .. } => check(run)?,
                    _ => {}
                }
            }
            Ok(())
        }

        check(&self.run)
    }

    pub(crate) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        for step in &mut self.run {
            step.resolve_devices(devices)?;
        }

        Ok(())
    }

    pub(crate) fn validate_capabilities(&self, registry: &DeviceRegistry) -> Result<(), String> {
        for step in &self.run {
            step.validate_capabilities(registry)?;
        }

        Ok(())
    }
}
