use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use crate::settings::adhoc_cron_task::AdhocCronTaskSettings;
use crate::settings::adhoc_tasks::AdhocTasksSettings;
use crate::settings::retention_parameters::RetentionParameters;
use sqlx::{Postgres, Transaction};

pub struct TrimWorkflowRuns;

#[async_trait::async_trait]
impl AdhocCronTask for TrimWorkflowRuns {
    type Parameters = RetentionParameters;

    fn name(&self) -> &'static str {
        "trim_workflow_runs"
    }

    fn settings<'a>(
        &self,
        tasks: &'a AdhocTasksSettings,
    ) -> &'a AdhocCronTaskSettings<Self::Parameters> {
        &tasks.trim_workflow_runs
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(
        &self,
        ctx: &mut AdhocTaskContext<'_>,
        parameters: &Self::Parameters,
    ) -> Result<u64, AdhocTaskError> {
        let batch_size = ctx.settings.adhoc.batch_size;

        Ok(trim(ctx.tx, batch_size, parameters.retention_secs()).await?)
    }
}

async fn trim(
    tx: &mut Transaction<'static, Postgres>,
    batch_size: i64,
    retention: f64,
) -> Result<u64, sqlx::Error> {
    let mut deleted = 0;

    loop {
        let batch = sqlx::query!(
            r#"
            DELETE FROM workflow_runs
            WHERE id IN (
                SELECT id
                FROM workflow_runs
                WHERE started_at < now() - make_interval(secs => $2)
                ORDER BY started_at
                LIMIT $1
            )
            "#,
            batch_size,
            retention,
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        deleted += batch;

        if batch < batch_size as u64 {
            break;
        }
    }

    Ok(deleted)
}
