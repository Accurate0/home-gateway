use crate::actors::system::cron::schedule::CronSchedule;

use super::context::AdhocTaskContext;
use super::error::AdhocTaskError;

#[async_trait::async_trait]
pub trait AdhocCronTask: Send + Sync {
    fn name(&self) -> &'static str;

    fn schedule(&self) -> CronSchedule;

    fn flag(&self) -> Option<&'static str> {
        None
    }

    fn source(&self) -> &'static str;

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError>;
}
