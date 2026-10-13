#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribution {
    Unchanged,
    Gateway,
    External { fields: Vec<&'static str> },
}
