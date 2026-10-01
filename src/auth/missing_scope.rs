use super::scope::Scope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingScope {
    pub scope: Scope,
}
