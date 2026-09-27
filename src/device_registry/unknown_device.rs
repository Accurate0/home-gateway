use super::id_or_alias::IdOrAlias;

#[derive(Debug, thiserror::Error)]
#[error("unknown device `{0}`")]
pub struct UnknownDevice(pub IdOrAlias);
