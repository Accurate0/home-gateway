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
    pub importance: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
}

impl UnraidVariables {
    pub fn new(subject: &str, kind: Option<DiskKind>, update: &UnraidUpdate) -> Self {
        let base = Self {
            subject: subject.to_owned(),
            kind: kind.map(|kind| kind.to_string()),
            change: None,
            state: None,
            previous: None,
            metric: None,
            value: None,
            importance: None,
            title: None,
            description: None,
            link: None,
        };

        match update {
            UnraidUpdate::Changed {
                change,
                state,
                previous,
            } => Self {
                change: Some(change.to_string()),
                state: Some(state.clone()),
                previous: Some(previous.clone()),
                ..base
            },
            UnraidUpdate::Reading { metric, value } => Self {
                metric: Some(metric.to_string()),
                value: Some(*value),
                ..base
            },
            UnraidUpdate::Notification {
                importance,
                title,
                description,
                link,
            } => Self {
                importance: Some(importance.to_string()),
                title: Some(title.clone()),
                description: Some(description.clone()),
                link: link.clone(),
                ..base
            },
        }
    }
}
