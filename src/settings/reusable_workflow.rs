use schemars::JsonSchema;
use serde::Deserialize;

use super::workflow::Step;
use super::workflow_context::ContextSource;
use super::{DeviceAliases, NotifyActionKind, validate_acknowledge, yes};
use crate::device_registry::DeviceRegistry;

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
    pub run: Vec<Step>,
}

impl ReusableWorkflow {
    pub fn template_placeholders(&self) -> Vec<String> {
        fn collect(steps: &[Step], out: &mut Vec<String>) {
            for step in steps {
                match step {
                    Step::Notify { message, title, .. } => {
                        out.extend(message.placeholders().into_iter().map(str::to_owned));
                        if let Some(title) = title {
                            out.extend(title.placeholders().into_iter().map(str::to_owned));
                        }
                    }
                    Step::MqttPublish { topic, payload, .. } => {
                        for template in [topic, payload] {
                            out.extend(template.placeholders().into_iter().map(str::to_owned));
                        }
                    }
                    Step::Http {
                        url, headers, body, ..
                    } => {
                        for template in std::iter::once(url).chain(headers.values()).chain(body) {
                            out.extend(template.placeholders().into_iter().map(str::to_owned));
                        }
                    }
                    Step::Scene { run, .. } => collect(run, out),
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        collect(&self.run, &mut out);
        out
    }

    pub fn run_workflow_targets(&self) -> Vec<&str> {
        fn collect<'a>(steps: &'a [Step], out: &mut Vec<&'a str>) {
            for step in steps {
                match step {
                    Step::RunWorkflow { workflow, .. } => out.push(workflow.as_str()),
                    Step::Scene { run, .. } => collect(run, out),
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        collect(&self.run, &mut out);
        out
    }

    pub fn notify_action_targets(&self) -> Vec<&str> {
        fn collect<'a>(steps: &'a [Step], out: &mut Vec<&'a str>) {
            for step in steps {
                match step {
                    Step::Notify { actions, .. } => {
                        for action in actions {
                            if let NotifyActionKind::RunWorkflow { workflow } = &action.action {
                                out.push(workflow.as_str());
                            }
                        }
                    }
                    Step::Scene { run, .. } => collect(run, out),
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        collect(&self.run, &mut out);
        out
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

    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        for step in &mut self.run {
            step.resolve_devices(devices)?;
        }

        Ok(())
    }

    pub(super) fn validate_capabilities(&self, registry: &DeviceRegistry) -> Result<(), String> {
        for step in &self.run {
            step.validate_capabilities(registry)?;
        }

        Ok(())
    }
}
