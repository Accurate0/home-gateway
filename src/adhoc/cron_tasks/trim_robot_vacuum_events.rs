use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use crate::settings::adhoc_cron_task::AdhocCronTaskSettings;
use crate::settings::adhoc_tasks::AdhocTasksSettings;
use crate::settings::retention_parameters::RetentionParameters;

pub struct TrimRobotVacuumEvents;

#[async_trait::async_trait]
impl AdhocCronTask for TrimRobotVacuumEvents {
    type Parameters = RetentionParameters;

    fn name(&self) -> &'static str {
        "trim_robot_vacuum_events"
    }

    fn settings<'a>(
        &self,
        tasks: &'a AdhocTasksSettings,
    ) -> &'a AdhocCronTaskSettings<Self::Parameters> {
        &tasks.trim_robot_vacuum_events
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(
        &self,
        ctx: &mut AdhocTaskContext<'_>,
        parameters: &Self::Parameters,
    ) -> Result<u64, AdhocTaskError> {
        let chunks = sqlx::query_scalar!(
            "SELECT drop_chunks('robot_vacuum_events', older_than => now() - make_interval(secs => $1))::text",
            parameters.retention_secs(),
        )
        .fetch_all(&mut **ctx.tx)
        .await?;

        Ok(chunks.len() as u64)
    }
}
