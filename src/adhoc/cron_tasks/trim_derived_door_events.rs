use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use crate::settings::adhoc_cron_task::AdhocCronTaskSettings;
use crate::settings::adhoc_tasks::AdhocTasksSettings;
use crate::settings::retention_parameters::RetentionParameters;
use sqlx::{Postgres, Transaction};

pub struct TrimDerivedDoorEvents;

#[async_trait::async_trait]
impl AdhocCronTask for TrimDerivedDoorEvents {
    type Parameters = RetentionParameters;

    fn name(&self) -> &'static str {
        "trim_derived_door_events"
    }

    fn settings<'a>(
        &self,
        tasks: &'a AdhocTasksSettings,
    ) -> &'a AdhocCronTaskSettings<Self::Parameters> {
        &tasks.trim_derived_door_events
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(
        &self,
        ctx: &mut AdhocTaskContext<'_>,
        parameters: &Self::Parameters,
    ) -> Result<u64, AdhocTaskError> {
        Ok(trim(ctx.tx, parameters.retention_secs()).await?)
    }
}

async fn trim(tx: &mut Transaction<'static, Postgres>, retention: f64) -> Result<u64, sqlx::Error> {
    let chunks = sqlx::query_scalar!(
        "SELECT drop_chunks('derived_door_events', older_than => now() - make_interval(secs => $1))::text",
        retention,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(chunks.len() as u64)
}
