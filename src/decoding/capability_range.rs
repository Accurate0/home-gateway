use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRange {
    pub min: u64,
    pub max: u64,
}

impl CapabilityRange {
    pub fn clamp(self, value: u64) -> u64 {
        value.clamp(self.min, self.max)
    }
}
