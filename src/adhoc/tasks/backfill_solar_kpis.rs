use sqlx::{Postgres, Transaction};

use crate::adhoc::{AdhocTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;

pub struct BackfillSolarKpis;

#[async_trait::async_trait]
impl AdhocTask for BackfillSolarKpis {
    fn ordinal(&self) -> i64 {
        2
    }

    fn name(&self) -> &'static str {
        "backfill_solar_kpis"
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<(), AdhocTaskError> {
        let batch_size = ctx.settings.adhoc.batch_size;

        Ok(backfill(ctx.tx, batch_size).await?)
    }
}

pub async fn backfill(
    tx: &mut Transaction<'static, Postgres>,
    batch_size: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!("SET LOCAL timescaledb.max_tuples_decompressed_per_dml_transaction = 0")
        .execute(&mut **tx)
        .await?;

    let mut total = 0;

    loop {
        let rows = sqlx::query!(
            "UPDATE solar_data_tsdb SET \
                 today_kwh = (raw_data -> 'data' -> 'kpi' ->> 'power')::float8, \
                 month_kwh = (raw_data -> 'data' -> 'kpi' ->> 'month_generation')::float8, \
                 total_kwh = (raw_data -> 'data' -> 'kpi' ->> 'total_power')::float8 \
             WHERE time IN ( \
                 SELECT time FROM solar_data_tsdb WHERE today_kwh IS NULL \
                 ORDER BY time DESC LIMIT $1 \
             )",
            batch_size
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        total += rows;
        tracing::info!("backfilled solar kpis for {rows} rows ({total} so far)");

        if rows < batch_size as u64 {
            break;
        }
    }

    Ok(())
}
