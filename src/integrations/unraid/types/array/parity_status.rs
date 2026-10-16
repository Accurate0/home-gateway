use strum::{Display, EnumString};

use crate::integrations::unraid::schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, cynic::Enum, Display, EnumString)]
#[cynic(graphql_type = "ParityCheckStatus")]
#[strum(serialize_all = "snake_case")]
pub enum ParityStatus {
    NeverRun,
    Running,
    Paused,
    Completed,
    Cancelled,
    Failed,
}
