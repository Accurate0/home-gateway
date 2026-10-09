pub mod conditions;
pub mod definition;
mod error;
pub mod manager;
pub mod mode;
pub mod plan;
pub mod timer_kind;
pub mod trace;
pub mod triggers;

pub use error::WorkflowError;

/// Maximum nesting depth for `run_workflow` expansion, guarding against
/// workflows that (directly or transitively) reference themselves.
pub const MAX_DEPTH: u8 = 8;
