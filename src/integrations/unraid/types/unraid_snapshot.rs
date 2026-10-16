use super::{ArraySnapshot, DiskSnapshot};

#[derive(Debug, Clone, PartialEq)]
pub struct UnraidSnapshot {
    pub array: ArraySnapshot,
    pub disks: Vec<DiskSnapshot>,
}
