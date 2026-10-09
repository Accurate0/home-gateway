use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(
    Clone,
    Debug,
    PartialEq,
    PartialOrd,
    sqlx::Type,
    serde::Serialize,
    serde::Deserialize,
    async_graphql::Enum,
    Eq,
    Copy,
    schemars::JsonSchema,
    strum::Display,
)]
#[sqlx(type_name = "air_purifier_mode", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AirPurifierMode {
    Manual,
    Sleep,
    Auto,
}

#[derive(Clone)]
pub struct AirPurifierRepo {
    db: Pool<Postgres>,
}

#[derive(Clone, Debug)]
pub struct AirPurifierStateRow {
    pub device_id: String,
    pub is_on: bool,
    pub mode: Option<AirPurifierMode>,
    pub speed: Option<i32>,
    pub pm25: Option<f64>,
    pub filter_life: Option<f64>,
    pub display: Option<bool>,
    pub changed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct AirPurifierRecord<'a> {
    pub event_id: Uuid,
    pub device_id: &'a str,
    pub is_on: bool,
    pub mode: Option<AirPurifierMode>,
    pub speed: Option<i32>,
    pub pm25: Option<f64>,
    pub filter_life: Option<f64>,
    pub display: Option<bool>,
}

impl AirPurifierRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.air_purifier.record", err)]
    pub async fn record(
        &self,
        record: &AirPurifierRecord<'_>,
    ) -> Result<Option<AirPurifierStateRow>, sqlx::Error> {
        let mut tx = self.db.begin().await?;

        let previous = sqlx::query_as!(
            AirPurifierStateRow,
            r#"
            SELECT device_id, is_on, mode AS "mode: AirPurifierMode", speed, pm25, filter_life,
                   display, changed_at, updated_at
            FROM latest_air_purifier_state
            WHERE device_id = $1
            FOR UPDATE
            "#,
            record.device_id,
        )
        .fetch_optional(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO air_purifier_events (event_id, device_id, is_on, mode, speed, pm25, filter_life, display) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            record.event_id,
            record.device_id,
            record.is_on,
            record.mode as Option<AirPurifierMode>,
            record.speed,
            record.pm25,
            record.filter_life,
            record.display,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO latest_air_purifier_state (device_id, is_on, mode, speed, pm25, filter_life, display) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (device_id) DO UPDATE SET \
               changed_at = CASE \
                 WHEN latest_air_purifier_state.is_on = EXCLUDED.is_on \
                   AND latest_air_purifier_state.mode IS NOT DISTINCT FROM COALESCE(EXCLUDED.mode, latest_air_purifier_state.mode) \
                   AND latest_air_purifier_state.speed IS NOT DISTINCT FROM COALESCE(EXCLUDED.speed, latest_air_purifier_state.speed) \
                 THEN latest_air_purifier_state.changed_at ELSE now() END, \
               is_on = EXCLUDED.is_on, \
               mode = COALESCE(EXCLUDED.mode, latest_air_purifier_state.mode), \
               speed = COALESCE(EXCLUDED.speed, latest_air_purifier_state.speed), \
               pm25 = COALESCE(EXCLUDED.pm25, latest_air_purifier_state.pm25), \
               filter_life = COALESCE(EXCLUDED.filter_life, latest_air_purifier_state.filter_life), \
               display = COALESCE(EXCLUDED.display, latest_air_purifier_state.display), \
               updated_at = now()",
            record.device_id,
            record.is_on,
            record.mode as Option<AirPurifierMode>,
            record.speed,
            record.pm25,
            record.filter_life,
            record.display,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(previous)
    }

    #[tracing::instrument(skip_all, name = "db.air_purifier.latest", err)]
    pub async fn latest(
        &self,
        device_id: &str,
    ) -> Result<Option<AirPurifierStateRow>, sqlx::Error> {
        sqlx::query_as!(
            AirPurifierStateRow,
            r#"
            SELECT device_id, is_on, mode AS "mode: AirPurifierMode", speed, pm25, filter_life,
                   display, changed_at, updated_at
            FROM latest_air_purifier_state
            WHERE device_id = $1
            "#,
            device_id
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.air_purifier.latest_many", fields(keys = keys.len()), err)]
    pub async fn latest_many(
        &self,
        keys: &[String],
    ) -> Result<Vec<AirPurifierStateRow>, sqlx::Error> {
        sqlx::query_as!(
            AirPurifierStateRow,
            r#"
            SELECT device_id, is_on, mode AS "mode: AirPurifierMode", speed, pm25, filter_life,
                   display, changed_at, updated_at
            FROM latest_air_purifier_state
            WHERE device_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .await
    }
}
