use sha2::{Digest, Sha256};

use super::context::AdhocTaskContext;
use super::error::AdhocTaskError;

#[async_trait::async_trait]
pub trait AdhocTask: Send + Sync {
    fn ordinal(&self) -> i64;

    fn name(&self) -> &'static str;

    fn flag(&self) -> Option<&'static str> {
        None
    }

    fn source(&self) -> &'static str;

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<(), AdhocTaskError>;
}

pub fn checksum(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());

    hex::encode(hasher.finalize())
}

#[macro_export]
macro_rules! adhoc_task_source {
    () => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", file!()))
    };
}
