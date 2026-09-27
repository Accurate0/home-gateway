#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdOrAlias(pub String);

impl std::fmt::Display for IdOrAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
