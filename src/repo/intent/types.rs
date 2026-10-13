use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::air_purifier_attributes::AirPurifierAttributes;
use crate::repo::air_purifier::AirPurifierState;
use crate::repo::light::{LightAttributes, LightState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Light,
    SmartSwitch,
    AirPurifier,
}

impl DeviceKind {
    pub const ALL: &'static [DeviceKind] = &[
        DeviceKind::Light,
        DeviceKind::SmartSwitch,
        DeviceKind::AirPurifier,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceKind::Light => "light",
            DeviceKind::SmartSwitch => "smart_switch",
            DeviceKind::AirPurifier => "air_purifier",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        DeviceKind::ALL
            .iter()
            .copied()
            .find(|kind| kind.as_str() == raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerState {
    On,
    Off,
}

impl PowerState {
    pub fn is_on(&self) -> bool {
        matches!(self, PowerState::On)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchAttributes {
    pub state: PowerState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IntentAttributes {
    Light(LightAttributes),
    SmartSwitch(SwitchAttributes),
    AirPurifier(AirPurifierAttributes),
}

impl IntentAttributes {
    pub fn kind(&self) -> DeviceKind {
        match self {
            IntentAttributes::Light(_) => DeviceKind::Light,
            IntentAttributes::SmartSwitch(_) => DeviceKind::SmartSwitch,
            IntentAttributes::AirPurifier(_) => DeviceKind::AirPurifier,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            IntentAttributes::Light(attributes) => attributes.is_empty(),
            IntentAttributes::SmartSwitch(_) => false,
            IntentAttributes::AirPurifier(attributes) => attributes.is_empty(),
        }
    }

    pub fn as_light(&self) -> Option<&LightAttributes> {
        match self {
            IntentAttributes::Light(attributes) => Some(attributes),
            IntentAttributes::SmartSwitch(_) | IntentAttributes::AirPurifier(_) => None,
        }
    }

    pub fn as_air_purifier(&self) -> Option<&AirPurifierAttributes> {
        match self {
            IntentAttributes::AirPurifier(attributes) => Some(attributes),
            IntentAttributes::Light(_) | IntentAttributes::SmartSwitch(_) => None,
        }
    }

    pub fn satisfied_by(&self, report: &DeviceReport) -> bool {
        match (self, report) {
            (IntentAttributes::Light(wanted), DeviceReport::Light(state)) => {
                wanted.satisfied_by(state)
            }
            (IntentAttributes::SmartSwitch(wanted), DeviceReport::Switch(state)) => {
                wanted.state == *state
            }
            (IntentAttributes::AirPurifier(wanted), DeviceReport::AirPurifier(state)) => {
                wanted.satisfied_by(state)
            }
            _ => false,
        }
    }

    pub fn explains(&self, field: &str, report: &DeviceReport) -> bool {
        match (self, report) {
            (IntentAttributes::Light(wanted), DeviceReport::Light(state)) => {
                wanted.explains(field, state)
            }
            (IntentAttributes::SmartSwitch(wanted), DeviceReport::Switch(state)) => {
                field == DeviceReport::SWITCH_STATE && wanted.state == *state
            }
            (IntentAttributes::AirPurifier(wanted), DeviceReport::AirPurifier(state)) => {
                wanted.explains(field, state)
            }
            _ => false,
        }
    }
}

pub enum DeviceReport<'a> {
    Light(&'a LightState),
    Switch(PowerState),
    AirPurifier(&'a AirPurifierState),
}

impl DeviceReport<'_> {
    pub const SWITCH_STATE: &'static str = "state";

    pub fn kind(&self) -> DeviceKind {
        match self {
            DeviceReport::Light(_) => DeviceKind::Light,
            DeviceReport::Switch(_) => DeviceKind::SmartSwitch,
            DeviceReport::AirPurifier(_) => DeviceKind::AirPurifier,
        }
    }

    pub fn changed_from(&self, previous: &DeviceReport) -> Vec<&'static str> {
        match (self, previous) {
            (DeviceReport::Light(current), DeviceReport::Light(previous)) => {
                current.changed_from(previous)
            }
            (DeviceReport::Switch(current), DeviceReport::Switch(previous)) => {
                if current == previous {
                    Vec::new()
                } else {
                    vec![Self::SWITCH_STATE]
                }
            }
            (DeviceReport::AirPurifier(current), DeviceReport::AirPurifier(previous)) => {
                current.changed_from(previous)
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentStatus {
    Pending,
    Confirmed,
    Failed,
    Superseded,
}

impl IntentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntentStatus::Pending => "pending",
            IntentStatus::Confirmed => "confirmed",
            IntentStatus::Failed => "failed",
            IntentStatus::Superseded => "superseded",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceIntent {
    pub id: i64,
    pub address: String,
    pub attributes: IntentAttributes,
    pub relative: bool,
    pub attempts: i32,
    pub event_id: Uuid,
    pub requested_at: DateTime<Utc>,
}

impl DeviceIntent {
    pub fn kind(&self) -> DeviceKind {
        self.attributes.kind()
    }
}
