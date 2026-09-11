use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use sqlx::{Postgres, Transaction};

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
