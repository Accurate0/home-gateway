use sqlx::PgPool;
use uuid::Uuid;

pub mod value;

pub use value::MetricValue;

pub struct DeviceMetric {
    pub event_id: Uuid,
    pub address: String,
    pub device_id: Option<String>,
    pub metric: String,
    pub value: MetricValue,
}

impl DeviceMetric {
    pub async fn save(&self, db: &PgPool) -> Result<(), sqlx::Error> {
        let (numeric, text) = self.value.columns();

        sqlx::query!(
            "INSERT INTO device_metric (event_id, address, device_id, metric, value, text_value) VALUES ($1, $2, $3, $4, $5, $6)",
            self.event_id,
            self.address,
            self.device_id,
            self.metric,
            numeric,
            text,
        )
        .execute(db)
        .await?;

        sqlx::query!(
            "INSERT INTO latest_device_metric (address, metric, device_id, value, text_value, updated_at) VALUES ($1, $2, $3, $4, $5, now()) \
             ON CONFLICT (address, metric) DO UPDATE SET device_id = EXCLUDED.device_id, value = EXCLUDED.value, text_value = EXCLUDED.text_value, updated_at = EXCLUDED.updated_at",
            self.address,
            self.metric,
            self.device_id,
            numeric,
            text,
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
