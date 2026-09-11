use schemars::JsonSchema;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

use super::{ReusableWorkflow, Workflow};
use crate::settings::DeviceAliases;

const TRIGGER_KEYS: &[&str] = &["modes", "when", "cooldown", "delay", "for"];

#[derive(Debug, Clone, JsonSchema)]
#[schemars(untagged)]
pub enum WorkflowDefinition {
    Triggered(Box<Workflow>),
    Reusable(ReusableWorkflow),
}

impl WorkflowDefinition {
    pub fn body(&self) -> &ReusableWorkflow {
        match self {
            WorkflowDefinition::Triggered(workflow) => &workflow.body,
            WorkflowDefinition::Reusable(workflow) => workflow,
        }
    }

    pub fn triggered(&self) -> Option<&Workflow> {
        match self {
            WorkflowDefinition::Triggered(workflow) => Some(workflow),
            WorkflowDefinition::Reusable(_) => None,
        }
    }

    pub(crate) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            WorkflowDefinition::Triggered(workflow) => workflow.resolve_devices(devices),
            WorkflowDefinition::Reusable(workflow) => workflow.resolve_devices(devices),
        }
    }
}

impl<'de> Deserialize<'de> for WorkflowDefinition {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;

        if value.get("on").is_some() {
            return serde_json::from_value(value)
                .map(WorkflowDefinition::Triggered)
                .map_err(D::Error::custom);
        }

        if let Some(key) = TRIGGER_KEYS.iter().find(|key| value.get(**key).is_some()) {
            return Err(D::Error::custom(format!("`{key}` needs an `on:` trigger")));
        }

        serde_json::from_value(value)
            .map(WorkflowDefinition::Reusable)
            .map_err(D::Error::custom)
    }
}
