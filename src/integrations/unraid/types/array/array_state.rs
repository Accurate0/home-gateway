use strum::{Display, EnumString};

use crate::integrations::unraid::schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, cynic::Enum, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ArrayState {
    Started,
    Stopped,
    NewArray,
    ReconDisk,
    DisableDisk,
    SwapDsbl,
    InvalidExpansion,
    ParityNotBiggest,
    TooManyMissingDisks,
    NewDiskTooSmall,
    NoDataDisks,
}
