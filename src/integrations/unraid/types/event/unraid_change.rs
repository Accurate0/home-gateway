use schemars::JsonSchema;
use serde::Deserialize;
use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, JsonSchema, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UnraidChange {
    ArrayState,
    DiskStatus,
    DiskErrors,
    ParityCheck,
}
