use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;

pub struct TrimLightHistory;

#[async_trait::async_trait]
impl AdhocCronTask for TrimLightHistory {
    fn name(&self) -> &'static str {
        "trim_light_history"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("50 3 * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        let chunks = sqlx::query_scalar!(
            "SELECT drop_chunks('light_history', older_than => now() - INTERVAL '90 days')::text"
        )
        .fetch_all(&mut **ctx.tx)
        .await?;

        Ok(chunks.len() as u64)
    }
}
