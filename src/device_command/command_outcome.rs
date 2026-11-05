#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, async_graphql::Enum)]
#[graphql(name = "HomeCommandOutcome")]
#[strum(serialize_all = "snake_case")]
pub enum CommandOutcome {
    Sent,
    Unchanged,
    Locked,
}
