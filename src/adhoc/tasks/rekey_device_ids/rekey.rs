#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rekey {
    pub from: &'static str,
    pub to: &'static str,
}

impl std::fmt::Display for Rekey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}
