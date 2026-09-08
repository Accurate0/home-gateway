use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use sqlx::{Postgres, Transaction};

const BATCH_SIZE: i64 = 10_000;

pub struct TrimDeviceIntent;

#[async_trait::async_trait]
impl AdhocCronTask for TrimDeviceIntent {
    fn name(&self) -> &'static str {
        "trim_device_intent"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("50 3 * * *").expect("valid cron")
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
            DELETE FROM device_intent
            WHERE id IN (
                SELECT id
                FROM device_intent
                WHERE status <> 'pending'
                  AND settled_at < now() - INTERVAL '14 days'
                ORDER BY settled_at
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
