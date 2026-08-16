use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use sqlx::{Postgres, Transaction};

const BATCH_SIZE: i64 = 10_000;

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
        Ok(trim(ctx.tx).await?)
    }
}

async fn trim(tx: &mut Transaction<'static, Postgres>) -> Result<u64, sqlx::Error> {
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
            BATCH_SIZE,
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        deleted += batch;

        if batch < BATCH_SIZE as u64 {
            break;
        }
    }

    Ok(deleted)
}
