use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct SolarRepo {
    db: Pool<Postgres>,
}

pub struct LatestSolarRow {
    pub raw_data: serde_json::Value,
    pub temperature: Option<f64>,
    pub uv_level: Option<f64>,
}

pub struct SolarReading {
    pub current_kwh: f64,
    pub today_kwh: f64,
    pub month_kwh: f64,
    pub total_kwh: f64,
    pub raw_data: serde_json::Value,
    pub uv_level: Option<f64>,
    pub temperature: Option<f64>,
}

pub struct LatestSolarKpis {
    pub current_kwh: f64,
    pub today_kwh: f64,
    pub month_kwh: f64,
    pub total_kwh: f64,
    pub uv_level: Option<f64>,
    pub temperature: Option<f64>,
}

pub struct SolarAveragesRow {
    pub last_15_mins: Option<f64>,
    pub last_1_hour: Option<f64>,
    pub last_3_hours: Option<f64>,
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

    #[tracing::instrument(skip_all, name = "db.solar.append_reading", err)]
    pub async fn append_reading(&self, reading: SolarReading) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO solar_data_tsdb \
                 (current_kwh, today_kwh, month_kwh, total_kwh, raw_data, uv_level, temperature) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            reading.current_kwh,
            reading.today_kwh,
            reading.month_kwh,
            reading.total_kwh,
            reading.raw_data,
            reading.uv_level,
            reading.temperature
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.solar.averages", err)]
    pub async fn averages(&self) -> Result<SolarAveragesRow, sqlx::Error> {
        sqlx::query_as!(
            SolarAveragesRow,
            "SELECT \
                 avg(current_kwh) FILTER (WHERE time > now() - INTERVAL '15 minutes') AS last_15_mins, \
                 avg(current_kwh) FILTER (WHERE time > now() - INTERVAL '1 hour') AS last_1_hour, \
                 avg(current_kwh) AS last_3_hours \
             FROM solar_data_tsdb WHERE time > now() - INTERVAL '3 hours'"
        )
        .fetch_one(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.solar.latest_kpis", err)]
    pub async fn latest_kpis(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Option<LatestSolarKpis>, sqlx::Error> {
        sqlx::query_as!(
            LatestSolarKpis,
            "SELECT current_kwh, today_kwh, month_kwh, total_kwh, uv_level, temperature \
             FROM solar_data_tsdb WHERE time > $1 ORDER BY time DESC LIMIT 1",
            since
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.solar.last_today_kwh_between", err)]
    pub async fn last_today_kwh_between(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Option<f64>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT today_kwh FROM solar_data_tsdb \
             WHERE time >= $1 AND time < $2 ORDER BY time DESC LIMIT 1",
            start,
            end
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| row.today_kwh))
    }

    #[tracing::instrument(skip_all, name = "db.solar.latest", err)]
    pub async fn latest(&self) -> Result<Option<LatestSolarRow>, sqlx::Error> {
        sqlx::query_as!(
            LatestSolarRow,
            "SELECT raw_data, temperature, uv_level FROM solar_data_tsdb ORDER BY time DESC LIMIT 1"
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.solar.buckets_since", err)]
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
        .await
    }

    #[tracing::instrument(skip_all, name = "db.solar.cached_token", err)]
    pub async fn cached_token(&self) -> Result<Option<CachedTokenRow>, sqlx::Error> {
        sqlx::query_as!(
            CachedTokenRow,
            "SELECT login_data, created_at FROM solar_cached_token ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_optional(&self.db)
        .await
    }
    #[tracing::instrument(skip_all, name = "db.solar.save_cached_token", err)]
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
