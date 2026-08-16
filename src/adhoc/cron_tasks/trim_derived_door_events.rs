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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use sqlx::Pool;

    async fn insert(pool: &Pool<Postgres>, id: &str, age_days: i32) {
        sqlx::query!(
            "INSERT INTO derived_door_events (event_id, name, id, ieee_addr, state, \"time\") \
             VALUES (gen_random_uuid(), $1, $1, '0x00', 'open', now() - make_interval(days => $2))",
            id,
            age_days,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn drops_chunks_past_the_cutoff_and_keeps_recent_ones(pool: Pool<Postgres>) {
        insert(&pool, "ancient", 800).await;
        insert(&pool, "recent", 1).await;

        let mut tx = pool.begin().await.unwrap();
        let chunks = trim(&mut tx).await.unwrap();
        tx.commit().await.unwrap();

        assert_eq!(chunks, 1);

        let remaining = sqlx::query_scalar!("SELECT id FROM derived_door_events")
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(remaining, vec!["recent"]);
    }
}
