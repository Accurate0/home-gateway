use super::IEEEAddress;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowEntityLightTypeState {
    On,
    Off,
    Toggle,
    IncreaseBrightness { value: u64 },
    DecreaseBrightness { value: u64 },
    IncreaseColourTemperature { value: u64 },
    DecreaseColourTemperature { value: u64 },
    StopColourTemperature,
    StopBrightness,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WorkflowEntityType {
    Light {
        #[serde(rename = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        #[serde(flatten)]
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
