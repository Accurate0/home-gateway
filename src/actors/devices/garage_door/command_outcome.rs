#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, async_graphql::Enum)]
#[strum(serialize_all = "snake_case")]
pub enum CommandOutcome {
    Sent,
    Unchanged,
}
