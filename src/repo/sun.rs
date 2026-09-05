use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct SunRepo {
    db: Pool<Postgres>,
}

impl SunRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn record_fired(
        &self,
        transition: &str,
        offset_seconds: i64,
        at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO sun_event_fired (transition, offset_seconds, fired_at) VALUES ($1, $2, $3) \
             ON CONFLICT (transition, offset_seconds) DO UPDATE SET fired_at = EXCLUDED.fired_at",
            transition,
            offset_seconds,
            at
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn last_fired(
        &self,
        transition: &str,
        offset_seconds: i64,
    ) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT fired_at FROM sun_event_fired WHERE transition = $1 AND offset_seconds = $2",
            transition,
            offset_seconds
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| row.fired_at))
    }
}
