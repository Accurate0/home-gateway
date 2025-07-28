#![allow(unused)]

use super::IEEEAddress;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowEntityLightTypeState {
    On,
    Off,
    Toggle,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WorkflowEntityType {
    Light {
        #[serde(rename = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        state: WorkflowEntityLightTypeState,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkflowSettings {
    pub run: Vec<WorkflowEntityType>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ActionSettings {
    pub workflow: WorkflowSettings,
}
