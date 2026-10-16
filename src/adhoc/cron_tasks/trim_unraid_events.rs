use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use crate::settings::adhoc_cron_task::AdhocCronTaskSettings;
use crate::settings::adhoc_tasks::AdhocTasksSettings;
use crate::settings::retention_parameters::RetentionParameters;

pub struct TrimUnraidEvents;

#[async_trait::async_trait]
impl AdhocCronTask for TrimUnraidEvents {
    type Parameters = RetentionParameters;

    fn name(&self) -> &'static str {
        "trim_unraid_events"
    }

    fn settings<'a>(
        &self,
        tasks: &'a AdhocTasksSettings,
    ) -> &'a AdhocCronTaskSettings<Self::Parameters> {
        &tasks.trim_unraid_events
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(
        &self,
        ctx: &mut AdhocTaskContext<'_>,
        parameters: &Self::Parameters,
    ) -> Result<u64, AdhocTaskError> {
        let array = sqlx::query_scalar!(
            "SELECT drop_chunks('unraid_array_events', older_than => now() - make_interval(secs => $1))::text",
            parameters.retention_secs(),
        )
        .fetch_all(&mut **ctx.tx)
        .await?;

        let disks = sqlx::query_scalar!(
            "SELECT drop_chunks('unraid_disk_events', older_than => now() - make_interval(secs => $1))::text",
            parameters.retention_secs(),
        )
        .fetch_all(&mut **ctx.tx)
        .await?;

        Ok((array.len() + disks.len()) as u64)
    }
}
