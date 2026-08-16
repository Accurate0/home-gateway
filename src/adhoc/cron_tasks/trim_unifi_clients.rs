use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;

pub struct TrimUnifiClients;

#[async_trait::async_trait]
impl AdhocCronTask for TrimUnifiClients {
    fn name(&self) -> &'static str {
        "trim_unifi_clients"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("10 3 * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        let chunks = sqlx::query_scalar!(
            "SELECT drop_chunks('unifi_clients', older_than => now() - INTERVAL '1 year')::text"
        )
        .fetch_all(&mut **ctx.tx)
        .await?;

        Ok(chunks.len() as u64)
    }
}
