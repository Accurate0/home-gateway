use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;

pub struct TrimRobotVacuumEvents;

#[async_trait::async_trait]
impl AdhocCronTask for TrimRobotVacuumEvents {
    fn name(&self) -> &'static str {
        "trim_robot_vacuum_events"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("40 3 * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        let chunks = sqlx::query_scalar!(
            "SELECT drop_chunks('robot_vacuum_events', older_than => now() - INTERVAL '180 days')::text"
        )
        .fetch_all(&mut **ctx.tx)
        .await?;

        Ok(chunks.len() as u64)
    }
}
