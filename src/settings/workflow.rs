use crate::settings::NotifySource;

use super::{DeviceAliases, IEEEAddress, resolve_device};
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
        // accepts a device alias (resolved at load time) or a raw address;
        // `ieeeAddr` kept as an alias for backwards compatibility with the HTTP execute route
        #[serde(rename = "device", alias = "ieeeAddr")]
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

impl WorkflowEntityType {
    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            WorkflowEntityType::Conditional { run, when } => {
                when.resolve_devices(devices)?;
                for step in run {
                    step.resolve_devices(devices)?;
                }
            }
            WorkflowEntityType::Light {
                ieee_addr, when, ..
            } => {
                *ieee_addr = resolve_device(ieee_addr, devices)?;
                if let Some(when) = when {
                    when.resolve_devices(devices)?;
                }
            }
            WorkflowEntityType::Notify { when, .. } => {
                if let Some(when) = when {
                    when.resolve_devices(devices)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WorkflowQueryType {
    Light {
        #[serde(rename = "device", alias = "ieeeAddr")]
        ieee_addr: IEEEAddress,
        #[serde(flatten)]
        state: WorkflowEntityLightQueryState,
    },
}

impl WorkflowQueryType {
    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        match self {
            WorkflowQueryType::Light { ieee_addr, .. } => {
                *ieee_addr = resolve_device(ieee_addr, devices)?;
            }
        }

        Ok(())
    }
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

/// On-disk form of an action: either a bare list of steps, or a struct with an
/// explicit `enabled` flag alongside `run`. Both collapse to [`ActionSettings`].
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum RawAction {
    Steps(Vec<WorkflowEntityType>),
    Detailed {
        #[serde(default = "yes")]
        enabled: bool,
        run: Vec<WorkflowEntityType>,
    },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(from = "RawAction")]
pub struct ActionSettings {
    pub workflow: WorkflowSettings,
}

impl From<RawAction> for ActionSettings {
    fn from(raw: RawAction) -> Self {
        let workflow = match raw {
            RawAction::Steps(run) => WorkflowSettings { enabled: true, run },
            RawAction::Detailed { enabled, run } => WorkflowSettings { enabled, run },
        };

        ActionSettings { workflow }
    }
}

impl ActionSettings {
    pub(super) fn resolve_devices(&mut self, devices: &DeviceAliases) -> Result<(), String> {
        for step in &mut self.workflow.run {
            step.resolve_devices(devices)?;
        }

        Ok(())
    }
}
