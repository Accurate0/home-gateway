use crate::settings::NotifySource;

use super::IEEEAddress;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowEntityLightTypeState {
    On,
    Off,
    Toggle,
    SetBrightness {
        value: u64,
    },
    IncreaseBrightness {
        value: u64,
        #[serde(default)]
        on_off: bool,
    },
    DecreaseBrightness {
        value: u64,
        #[serde(default)]
        on_off: bool,
    },
    IncreaseColourTemperature {
        value: u64,
    },
    DecreaseColourTemperature {
        value: u64,
    },
    StopColourTemperature,
    StopBrightness,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowEntityLightQueryState {
    On,
    Off,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WorkflowEntityType {
    Conditional {
        run: Vec<WorkflowEntityType>,
        when: WorkflowQueryType,
    },
    Light {
        #[serde(rename = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        #[serde(flatten)]
        state: WorkflowEntityLightTypeState,
        #[serde(default)]
        when: Option<WorkflowQueryType>,
    },
    Notify {
        #[serde(flatten)]
        notify: NotifySource,
        message: String,
        when: Option<WorkflowQueryType>,
    },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WorkflowQueryType {
    Light {
        #[serde(rename = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        #[serde(flatten)]
        state: WorkflowEntityLightQueryState,
    },
}

fn yes() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkflowSettings {
    #[serde(default = "yes")]
    pub enabled: bool,
    pub run: Vec<WorkflowEntityType>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ActionSettings {
    pub workflow: WorkflowSettings,
}
