use crate::integrations::unraid::types::{DiskKind, UnraidUpdate};
use crate::variables::WorkflowContextVariables;

#[derive(WorkflowContextVariables)]
pub struct UnraidVariables {
    pub subject: String,
    pub kind: Option<String>,
    pub change: Option<String>,
    pub state: Option<String>,
    pub previous: Option<String>,
    pub metric: Option<String>,
    pub value: Option<f64>,
}

impl UnraidVariables {
    pub fn new(subject: &str, kind: Option<DiskKind>, update: &UnraidUpdate) -> Self {
        let kind = kind.map(|kind| kind.to_string());

        match update {
            UnraidUpdate::Changed {
                change,
                state,
                previous,
            } => Self {
                subject: subject.to_owned(),
                kind,
                change: Some(change.to_string()),
                state: Some(state.clone()),
                previous: Some(previous.clone()),
                metric: None,
                value: None,
            },
            UnraidUpdate::Reading { metric, value } => Self {
                subject: subject.to_owned(),
                kind,
                change: None,
                state: None,
                previous: None,
                metric: Some(metric.to_string()),
                value: Some(*value),
            },
        }
    }
}
