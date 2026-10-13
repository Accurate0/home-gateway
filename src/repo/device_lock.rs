use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    sqlx::Type,
    serde::Serialize,
    async_graphql::Enum,
    strum::Display,
)]
#[sqlx(type_name = "device_lock_source", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DeviceLockSource {
    Physical,
    Manual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeviceLock {
    pub device_id: String,
    pub source: DeviceLockSource,
    pub locked_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct DeviceLockRepo {
    db: Pool<Postgres>,
}

impl DeviceLockRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.device_lock.active", err)]
    pub async fn active(&self, device_id: &str) -> Result<Option<DeviceLock>, sqlx::Error> {
        sqlx::query_as!(
            DeviceLock,
            r#"
            SELECT device_id, source AS "source: DeviceLockSource", locked_at, expires_at
            FROM device_lock
            WHERE device_id = $1 AND (expires_at IS NULL OR expires_at > now())
            "#,
            device_id
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.device_lock.active_many", fields(keys = keys.len()), err)]
    pub async fn active_many(&self, keys: &[String]) -> Result<Vec<DeviceLock>, sqlx::Error> {
        sqlx::query_as!(
            DeviceLock,
            r#"
            SELECT device_id, source AS "source: DeviceLockSource", locked_at, expires_at
            FROM device_lock
            WHERE device_id = ANY($1) AND (expires_at IS NULL OR expires_at > now())
            "#,
            keys
        )
        .fetch_all(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.device_lock.replace", err)]
    pub async fn replace(
        &self,
        device_id: &str,
        source: DeviceLockSource,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<DeviceLock, sqlx::Error> {
        sqlx::query_as!(
            DeviceLock,
            r#"
            INSERT INTO device_lock (device_id, source, locked_at, expires_at)
            VALUES ($1, $2, now(), $3)
            ON CONFLICT (device_id) DO UPDATE SET
              source = EXCLUDED.source,
              locked_at = EXCLUDED.locked_at,
              expires_at = EXCLUDED.expires_at
            RETURNING device_id, source AS "source: DeviceLockSource", locked_at, expires_at
            "#,
            device_id,
            source as DeviceLockSource,
            expires_at,
        )
        .fetch_one(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.device_lock.extend", err)]
    pub async fn extend(
        &self,
        device_id: &str,
        source: DeviceLockSource,
        expires_at: DateTime<Utc>,
    ) -> Result<Option<DeviceLock>, sqlx::Error> {
        sqlx::query_as!(
            DeviceLock,
            r#"
            INSERT INTO device_lock (device_id, source, locked_at, expires_at)
            VALUES ($1, $2, now(), $3)
            ON CONFLICT (device_id) DO UPDATE SET
              source = EXCLUDED.source,
              locked_at = EXCLUDED.locked_at,
              expires_at = EXCLUDED.expires_at
            WHERE device_lock.expires_at IS NOT NULL
              AND device_lock.expires_at < EXCLUDED.expires_at
            RETURNING device_id, source AS "source: DeviceLockSource", locked_at, expires_at
            "#,
            device_id,
            source as DeviceLockSource,
            expires_at,
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.device_lock.remove", err)]
    pub async fn remove(&self, device_id: &str) -> Result<bool, sqlx::Error> {
        let removed = sqlx::query!(
            "DELETE FROM device_lock \
             WHERE device_id = $1 AND (expires_at IS NULL OR expires_at > now())",
            device_id
        )
        .execute(&self.db)
        .await?;

        Ok(removed.rows_affected() > 0)
    }
}
