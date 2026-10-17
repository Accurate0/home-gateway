use super::{DiskSerials, UnraidSnapshot};

#[derive(Debug, Clone, PartialEq)]
pub struct UnraidPoll {
    pub snapshot: UnraidSnapshot,
    pub serials: DiskSerials,
}
