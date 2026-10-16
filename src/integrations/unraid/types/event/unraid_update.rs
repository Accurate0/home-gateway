use super::{UnraidChange, UnraidMetric};

#[derive(Debug, Clone, PartialEq)]
pub enum UnraidUpdate {
    Changed {
        change: UnraidChange,
        state: String,
        previous: String,
    },
    Reading {
        metric: UnraidMetric,
        value: f64,
    },
}
