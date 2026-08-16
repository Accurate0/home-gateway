#[derive(Debug, thiserror::Error)]
pub enum AdhocTaskError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("task failed: {0}")]
    Failed(String),

    #[error("task panicked")]
    Panicked,

    #[error("task timed out")]
    TimedOut,
}
