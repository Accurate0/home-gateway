use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;

pub struct TrimHomeAssistantEvents;

#[async_trait::async_trait]
impl AdhocCronTask for TrimHomeAssistantEvents {
    fn name(&self) -> &'static str {
        "trim_home_assistant_events"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("25 3 * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        let chunks = sqlx::query_scalar!(
            "SELECT drop_chunks('home_assistant_events', older_than => now() - INTERVAL '90 days')::text"
        )
        .fetch_all(&mut **ctx.tx)
        .await?;

        Ok(chunks.len() as u64)
    }
}
