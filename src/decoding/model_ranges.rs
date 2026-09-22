use serde::Deserialize;

use super::capability_range::CapabilityRange;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelRanges {
    pub brightness: Option<CapabilityRange>,
    pub colour_temp: Option<CapabilityRange>,
}
