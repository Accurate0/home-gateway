use crate::actors::system::cron::schedule::CronSchedule;
use crate::adhoc::{AdhocCronTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use sqlx::{Postgres, Transaction};

pub struct TrimDerivedDoorEvents;

#[async_trait::async_trait]
impl AdhocCronTask for TrimDerivedDoorEvents {
    fn name(&self) -> &'static str {
        "trim_derived_door_events"
    }

    fn schedule(&self) -> CronSchedule {
        CronSchedule::parse("0 3 * * *").expect("valid cron")
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<u64, AdhocTaskError> {
        Ok(trim(ctx.tx).await?)
    }
}

async fn trim(tx: &mut Transaction<'static, Postgres>) -> Result<u64, sqlx::Error> {
    let chunks = sqlx::query_scalar!(
        "SELECT drop_chunks('derived_door_events', older_than => now() - INTERVAL '1 year')::text"
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(chunks.len() as u64)
}
