use schemars::JsonSchema;
use serde::Deserialize;

use super::Condition;
use super::condition::describe_join;
use crate::settings::DeviceAliases;

#[derive(Debug, Deserialize, Clone, JsonSchema)]
pub enum Combinator {
    #[serde(rename = "all", alias = "and")]
    All(Vec<Condition>),
    #[serde(rename = "any", alias = "or")]
    Any(Vec<Condition>),
    #[serde(rename = "not")]
    Not(Box<Condition>),
}

impl Combinator {
    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            Combinator::All(conditions) | Combinator::Any(conditions) => {
                for c in conditions {
                    c.resolve_devices(devices)?;
                }
            }
            Combinator::Not(condition) => condition.resolve_devices(devices)?,
        }
        Ok(())
    }

    pub(super) fn describe(&self) -> String {
        match self {
            Combinator::All(conditions) => format!("all[{}]", describe_join(conditions)),
            Combinator::Any(conditions) => format!("any[{}]", describe_join(conditions)),
            Combinator::Not(condition) => format!("not({})", condition.describe()),
        }
    }
}
