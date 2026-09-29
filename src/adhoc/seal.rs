use std::fmt;

use super::task::AdhocTask;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seal {
    Locked,
    Unlocked,
}

impl fmt::Display for Seal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Seal::Locked => write!(f, "locked"),
            Seal::Unlocked => write!(f, "unlocked"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SealedTask {
    pub task: &'static dyn AdhocTask,
    pub seal: Seal,
}

impl SealedTask {
    pub const fn locked(task: &'static dyn AdhocTask) -> Self {
        Self {
            task,
            seal: Seal::Locked,
        }
    }

    pub const fn unlocked(task: &'static dyn AdhocTask) -> Self {
        Self {
            task,
            seal: Seal::Unlocked,
        }
    }
}
