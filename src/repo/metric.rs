use crate::device_metric::DeviceMetric;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct MetricRepo {
    db: Pool<Postgres>,
}

impl MetricRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn record(&self, metric: &DeviceMetric) -> Result<(), sqlx::Error> {
        let (numeric, text) = metric.value.columns();
        let mut tx = self.db.begin().await?;

        sqlx::query!(
            "INSERT INTO device_metric (event_id, address, device_id, metric, value, text_value) VALUES ($1, $2, $3, $4, $5, $6)",
            metric.event_id,
            metric.address,
            metric.device_id,
            metric.metric,
            numeric,
            text,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO latest_device_metric (address, metric, device_id, value, text_value, updated_at) VALUES ($1, $2, $3, $4, $5, now()) \
             ON CONFLICT (address, metric) DO UPDATE SET device_id = EXCLUDED.device_id, value = EXCLUDED.value, text_value = EXCLUDED.text_value, updated_at = EXCLUDED.updated_at",
            metric.address,
            metric.metric,
            metric.device_id,
            numeric,
            text,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }
}
