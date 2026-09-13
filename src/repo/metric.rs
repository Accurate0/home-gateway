use crate::device_metric::DeviceMetric;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct MetricRepo {
    db: Pool<Postgres>,
}

pub struct MetricRow {
    pub value: Option<f64>,
    pub text_value: Option<String>,
    pub time: DateTime<Utc>,
}

impl MetricRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.metric.record", err)]
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

    #[tracing::instrument(skip_all, name = "db.metric.latest", err)]
    pub async fn latest(
        &self,
        address: &str,
        metric: &str,
    ) -> Result<Option<MetricRow>, sqlx::Error> {
        sqlx::query_as!(
            MetricRow,
            r#"SELECT value, text_value, updated_at AS "time!"
               FROM latest_device_metric
               WHERE address = $1 AND metric = $2"#,
            address,
            metric,
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.metric.since", err)]
    pub async fn since(
        &self,
        address: &str,
        metric: &str,
        earliest: DateTime<Utc>,
    ) -> Result<Vec<MetricRow>, sqlx::Error> {
        sqlx::query_as!(
            MetricRow,
            r#"SELECT value, text_value, "time"
               FROM device_metric
               WHERE address = $1 AND metric = $2 AND "time" >= $3
               ORDER BY "time" ASC"#,
            address,
            metric,
            earliest,
        )
        .fetch_all(&self.db)
        .await
    }
}
