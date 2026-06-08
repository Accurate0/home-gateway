use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use std::fmt::Display;

use crate::actors::reminder::cronlike_expression::{
    CronlikeExpression, cronlike_expression_from_str,
};

use super::notify::{NotifyRef, NotifySource, NotifyTargets, resolve_notify};

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReminderState {
    DryRun,
    Active,
}

impl Display for ReminderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReminderState::DryRun => write!(f, "dryrun"),
            ReminderState::Active => write!(f, "active"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReminderSettings {
    #[allow(unused)]
    pub id: String,
    pub name: String,
    pub state: ReminderState,
    pub starts_on: DateTime<FixedOffset>,
    pub frequency: CronlikeExpression,
    pub notify: Vec<NotifySource>,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct RawReminderSettings {
    id: String,
    name: String,
    state: ReminderState,
    starts_on: DateTime<FixedOffset>,
    #[serde(with = "cronlike_expression_from_str")]
    frequency: CronlikeExpression,
    #[serde(default)]
    notify: Vec<NotifyRef>,
}

impl RawReminderSettings {
    pub(super) fn resolve(self, targets: &NotifyTargets) -> Result<ReminderSettings, String> {
        Ok(ReminderSettings {
            id: self.id,
            name: self.name,
            state: self.state,
            starts_on: self.starts_on,
            frequency: self.frequency,
            notify: resolve_notify(self.notify, targets)?,
        })
    }
}
