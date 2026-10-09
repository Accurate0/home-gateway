use schemars::JsonSchema;
use serde::Deserialize;

use super::CompareOp;

/// A scalar comparison: `{ op: gt, value: 30 }`. Flattened into the
/// [`super::LeafCondition::Environment`] variant.
#[derive(Debug, Deserialize, Clone, Copy, JsonSchema)]
pub struct Comparison {
    pub op: CompareOp,
    pub value: f64,
}

impl Comparison {
    pub fn matches(&self, actual: f64) -> bool {
        match self.op {
            CompareOp::Gt => actual > self.value,
            CompareOp::Lt => actual < self.value,
            CompareOp::Gte => actual >= self.value,
            CompareOp::Lte => actual <= self.value,
            // direct float equality is intentional: thresholds are configured as
            // exact values and sensors report discrete readings
            CompareOp::Eq => actual == self.value,
        }
    }
}
