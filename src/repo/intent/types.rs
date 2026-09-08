use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repo::light::{LightAttributes, LightState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Light,
    SmartSwitch,
}

impl DeviceKind {
    pub const ALL: &'static [DeviceKind] = &[DeviceKind::Light, DeviceKind::SmartSwitch];

    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceKind::Light => "light",
            DeviceKind::SmartSwitch => "smart_switch",
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
}

impl IntentAttributes {
    pub fn kind(&self) -> DeviceKind {
        match self {
            IntentAttributes::Light(_) => DeviceKind::Light,
            IntentAttributes::SmartSwitch(_) => DeviceKind::SmartSwitch,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            IntentAttributes::Light(attributes) => attributes.is_empty(),
            IntentAttributes::SmartSwitch(_) => false,
        }
    }

    pub fn as_light(&self) -> Option<&LightAttributes> {
        match self {
            IntentAttributes::Light(attributes) => Some(attributes),
            IntentAttributes::SmartSwitch(_) => None,
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
            _ => false,
        }
    }
}

pub enum DeviceReport<'a> {
    Light(&'a LightState),
    Switch(PowerState),
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
