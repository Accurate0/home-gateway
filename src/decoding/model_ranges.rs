use serde::Deserialize;

use super::mired_range::MiredRange;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelRanges {
    pub colour_temp: Option<MiredRange>,
}
