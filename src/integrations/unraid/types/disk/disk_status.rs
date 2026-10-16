use strum::{Display, EnumString};

use crate::integrations::unraid::schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, cynic::Enum, Display, EnumString)]
#[cynic(graphql_type = "ArrayDiskStatus")]
#[strum(serialize_all = "snake_case")]
pub enum DiskStatus {
    DiskNp,
    DiskOk,
    DiskNpMissing,
    DiskInvalid,
    DiskWrong,
    DiskDsbl,
    DiskNpDsbl,
    DiskDsblNew,
    DiskNew,
}
