use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

pub struct StoredRender {
    pub image_key: Option<String>,
    pub image_content_hash: Option<String>,
}

pub struct RtcReportRecord {
    pub reported_at: DateTime<Utc>,
    pub offset_ms: i64,
}

pub struct RtcSyncRecord {
    pub synced_at: DateTime<Utc>,
    pub drift_ms: Option<i64>,
}

pub struct WakeRecord<'a> {
    pub next_wake_at: DateTime<Utc>,
    pub displayed_hash: Option<Option<&'a str>>,
    pub rtc_report: Option<RtcReportRecord>,
    pub rtc_sync: Option<RtcSyncRecord>,
}

#[derive(Clone)]
pub struct EinkRepo {
    db: Pool<Postgres>,
}

#[derive(Clone)]
pub struct EinkDisplayRow {
    pub device_id: String,
    pub battery_voltage: Option<f64>,
    pub is_charging: Option<bool>,
    pub updated_at: DateTime<Utc>,
    pub next_wake_at: Option<DateTime<Utc>>,
    pub partial_refresh_count: i32,
    pub rtc_synced_at: Option<DateTime<Utc>>,
    pub rtc_drift_ms: Option<i64>,
    pub rtc_reported_at: Option<DateTime<Utc>>,
    pub rtc_reported_offset_ms: Option<i64>,
    pub last_wake_trace_id: Option<String>,
}

impl EinkRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.eink.store_render", err)]
    pub async fn store_render(
        &self,
        device_id: &str,
        name: &str,
        image_key: &str,
        content_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, image_key, image_content_hash) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, image_key = EXCLUDED.image_key, image_content_hash = EXCLUDED.image_content_hash",
            device_id,
            name,
            image_key,
            content_hash,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.eink.store_seen", err)]
    pub async fn store_seen(
        &self,
        device_id: &str,
        name: &str,
        trace_id: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, updated_at, last_wake_trace_id) VALUES ($1, $2, now(), $3) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, updated_at = EXCLUDED.updated_at, last_wake_trace_id = EXCLUDED.last_wake_trace_id",
            device_id,
            name,
            trace_id,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.eink.store_battery", err)]
    pub async fn store_battery(
        &self,
        device_id: &str,
        name: &str,
        battery_voltage: f64,
        is_charging: Option<bool>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, battery_voltage, is_charging, updated_at) VALUES ($1, $2, $3, $4, now()) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, battery_voltage = EXCLUDED.battery_voltage, is_charging = EXCLUDED.is_charging, updated_at = EXCLUDED.updated_at",
            device_id,
            name,
            battery_voltage,
            is_charging,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.eink.store_next_wake", err)]
    pub async fn store_next_wake(
        &self,
        device_id: &str,
        name: &str,
        next_wake_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, next_wake_at) VALUES ($1, $2, $3) \
             ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, next_wake_at = EXCLUDED.next_wake_at",
            device_id,
            name,
            next_wake_at,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.eink.stored_next_wake", err)]
    pub async fn stored_next_wake(
        &self,
        device_id: &str,
    ) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT next_wake_at FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.and_then(|row| row.next_wake_at))
    }

    #[tracing::instrument(skip_all, name = "db.eink.displayed_hash", err)]
    pub async fn displayed_hash(&self, device_id: &str) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT displayed_hash FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.and_then(|row| row.displayed_hash))
    }

    #[tracing::instrument(skip_all, name = "db.eink.store_wake", err)]
    pub async fn store_wake(
        &self,
        device_id: &str,
        name: &str,
        wake: &WakeRecord<'_>,
    ) -> Result<(), sqlx::Error> {
        let rtc_report = wake.rtc_report.as_ref();
        let rtc_sync = wake.rtc_sync.as_ref();

        sqlx::query!(
            "INSERT INTO eink_display (device_id, name, next_wake_at, displayed_hash, rtc_reported_at, rtc_reported_offset_ms, rtc_synced_at, rtc_drift_ms) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (device_id) DO UPDATE SET \
                 name = EXCLUDED.name, \
                 next_wake_at = EXCLUDED.next_wake_at, \
                 displayed_hash = CASE WHEN $9::boolean THEN EXCLUDED.displayed_hash ELSE eink_display.displayed_hash END, \
                 rtc_reported_at = CASE WHEN $10::boolean THEN EXCLUDED.rtc_reported_at ELSE eink_display.rtc_reported_at END, \
                 rtc_reported_offset_ms = CASE WHEN $10::boolean THEN EXCLUDED.rtc_reported_offset_ms ELSE eink_display.rtc_reported_offset_ms END, \
                 rtc_synced_at = CASE WHEN $11::boolean THEN EXCLUDED.rtc_synced_at ELSE eink_display.rtc_synced_at END, \
                 rtc_drift_ms = CASE WHEN $11::boolean THEN EXCLUDED.rtc_drift_ms ELSE eink_display.rtc_drift_ms END",
            device_id,
            name,
            wake.next_wake_at,
            wake.displayed_hash.flatten(),
            rtc_report.map(|report| report.reported_at),
            rtc_report.map(|report| report.offset_ms),
            rtc_sync.map(|sync| sync.synced_at),
            rtc_sync.and_then(|sync| sync.drift_ms),
            wake.displayed_hash.is_some(),
            rtc_report.is_some(),
            rtc_sync.is_some(),
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.eink.rtc_synced_at", err)]
    pub async fn rtc_synced_at(
        &self,
        device_id: &str,
    ) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT rtc_synced_at FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.and_then(|row| row.rtc_synced_at))
    }

    #[tracing::instrument(skip_all, name = "db.eink.stored_render", err)]
    pub async fn stored_render(
        &self,
        device_id: &str,
    ) -> Result<Option<StoredRender>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT image_key, image_content_hash FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| StoredRender {
            image_key: row.image_key,
            image_content_hash: row.image_content_hash,
        }))
    }

    #[tracing::instrument(skip_all, name = "db.eink.partial_refresh_count", err)]
    pub async fn partial_refresh_count(&self, device_id: &str) -> Result<i32, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT partial_refresh_count FROM eink_display WHERE device_id = $1",
            device_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(count.unwrap_or(0))
    }

    #[tracing::instrument(skip_all, name = "db.eink.increment_partial_refresh_count", err)]
    pub async fn increment_partial_refresh_count(
        &self,
        device_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE eink_display SET partial_refresh_count = partial_refresh_count + 1 WHERE device_id = $1",
            device_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.eink.reset_partial_refresh_count", err)]
    pub async fn reset_partial_refresh_count(&self, device_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE eink_display SET partial_refresh_count = 0 WHERE device_id = $1",
            device_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
    #[tracing::instrument(skip_all, name = "db.eink.battery_many", fields(keys = keys.len()), err)]
    pub async fn battery_many(&self, keys: &[String]) -> Result<Vec<EinkDisplayRow>, sqlx::Error> {
        sqlx::query_as!(
            EinkDisplayRow,
            r#"
            SELECT device_id, battery_voltage, is_charging, updated_at,
                   next_wake_at, partial_refresh_count, rtc_synced_at, rtc_drift_ms,
                   rtc_reported_at, rtc_reported_offset_ms, last_wake_trace_id
            FROM eink_display
            WHERE device_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .await
    }
}
