use strum::{Display, EnumString};

use crate::integrations::unraid::schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, cynic::Enum, Display, EnumString)]
#[cynic(graphql_type = "ArrayDiskType")]
#[strum(serialize_all = "snake_case")]
pub enum DiskKind {
    Data,
    Parity,
    Boot,
    Flash,
    Cache,
}
