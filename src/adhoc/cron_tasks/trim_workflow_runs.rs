use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use sqlx::{Postgres, Transaction};

pub struct TrimWorkflowRuns;

#[async_trait::async_trait]
impl AdhocCronTask for TrimWorkflowRuns {
    fn name(&self) -> &'static str {
        "trim_workflow_runs"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("45 3 * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        let batch_size = ctx.settings.adhoc.batch_size;

        Ok(trim(ctx.tx, batch_size).await?)
    }
}

async fn trim(
    tx: &mut Transaction<'static, Postgres>,
    batch_size: i64,
) -> Result<u64, sqlx::Error> {
    let mut deleted = 0;

    loop {
        let batch = sqlx::query!(
            r#"
            DELETE FROM workflow_runs
            WHERE id IN (
                SELECT id
                FROM workflow_runs
                WHERE started_at < now() - INTERVAL '90 days'
                ORDER BY started_at
                LIMIT $1
            )
            "#,
            batch_size,
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
