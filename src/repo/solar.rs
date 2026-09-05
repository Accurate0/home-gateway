use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use tracing::Instrument;

#[derive(Clone)]
pub struct SolarRepo {
    db: Pool<Postgres>,
}

pub struct LatestSolarRow {
    pub raw_data: serde_json::Value,
    pub temperature: Option<f64>,
    pub uv_level: Option<f64>,
}

pub struct SolarBucketRow {
    pub avg_wh: Option<f64>,
    pub avg_uv_level: Option<f64>,
    pub avg_temp: Option<f64>,
    pub bucket_time: Option<DateTime<Utc>>,
}

pub struct CachedTokenRow {
    pub login_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl SolarRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn append_reading(
        &self,
        current_kwh: f64,
        raw_data: serde_json::Value,
        uv_level: Option<f64>,
        temperature: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO solar_data_tsdb (current_kwh, raw_data, uv_level, temperature) \
             VALUES ($1, $2, $3, $4)",
            current_kwh,
            raw_data,
            uv_level,
            temperature
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
    pub async fn average_for_last_n_minutes(
        &self,
        minutes: i32,
    ) -> Result<Option<f64>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT avg(current_kwh) AS avg FROM solar_data_tsdb \
             WHERE time > now() - MAKE_INTERVAL(mins => $1)",
            minutes
        )
        .fetch_optional(&self.db)
        .instrument(tracing::info_span!("solar_average", time_in_mins = minutes))
        .await?;

        Ok(row.and_then(|row| row.avg))
    }

    pub async fn latest(&self) -> Result<Option<LatestSolarRow>, sqlx::Error> {
        sqlx::query_as!(
            LatestSolarRow,
            "SELECT raw_data, temperature, uv_level FROM solar_data_tsdb ORDER BY time DESC LIMIT 1"
        )
        .fetch_optional(&self.db)
        .instrument(tracing::info_span!("get_latest_solar_data"))
        .await
    }

    pub async fn yesterday_raw_data(&self) -> Result<Option<serde_json::Value>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT raw_data FROM solar_data_tsdb \
             WHERE (time AT TIME ZONE 'Australia/Perth')::date \
                 = (now() AT TIME ZONE 'Australia/Perth')::date - INTEGER '1' \
             ORDER BY time DESC LIMIT 1"
        )
        .fetch_optional(&self.db)
        .instrument(tracing::info_span!("get_yesterday_results"))
        .await?;

        Ok(row.map(|row| row.raw_data))
    }

    pub async fn buckets_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<SolarBucketRow>, sqlx::Error> {
        sqlx::query_as!(
            SolarBucketRow,
            "SELECT avg(current_kwh) AS avg_wh, avg(uv_level) AS avg_uv_level, \
                    avg(temperature) AS avg_temp, time_bucket('5 minutes', time) AS bucket_time \
             FROM solar_data_tsdb WHERE time >= $1 \
             GROUP BY bucket_time ORDER BY bucket_time ASC",
            since
        )
        .fetch_all(&self.db)
        .instrument(tracing::info_span!("solar_history_since"))
        .await
    }

    pub async fn buckets_last_two_days(&self) -> Result<Vec<SolarBucketRow>, sqlx::Error> {
        sqlx::query_as!(
            SolarBucketRow,
            "SELECT avg(current_kwh) AS avg_wh, avg(uv_level) AS avg_uv_level, \
                    avg(temperature) AS avg_temp, time_bucket('5 minutes', time) AS bucket_time \
             FROM solar_data_tsdb \
             WHERE (time AT TIME ZONE 'Australia/Perth')::date \
                 > ((now() AT TIME ZONE 'Australia/Perth')::date - 2) \
             GROUP BY bucket_time ORDER BY bucket_time ASC"
        )
        .fetch_all(&self.db)
        .instrument(tracing::info_span!("solar_history_two_days"))
        .await
    }

    pub async fn cached_token(&self) -> Result<Option<CachedTokenRow>, sqlx::Error> {
        sqlx::query_as!(
            CachedTokenRow,
            "SELECT login_data, created_at FROM solar_cached_token ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_optional(&self.db)
        .await
    }
    pub async fn save_cached_token(
        &self,
        login_data: serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO solar_cached_token (login_data) VALUES ($1)",
            login_data
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
