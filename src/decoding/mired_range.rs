use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiredRange {
    pub min: u64,
    pub max: u64,
}

impl MiredRange {
    pub fn clamp(self, mireds: u64) -> u64 {
        mireds.clamp(self.min, self.max)
    }
}
