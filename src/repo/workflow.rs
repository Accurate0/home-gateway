use chrono::{DateTime, TimeDelta, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub struct NewWorkflowRun<'a> {
    pub slug: &'a str,
    pub name: &'a str,
    pub event_id: Uuid,
    pub outcome: &'a str,
    pub dry_run: bool,
    pub duration_ms: i64,
    pub error: Option<&'a str>,
}

pub struct WorkflowRunRow {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub event_id: Uuid,
    pub outcome: String,
    pub dry_run: bool,
    pub duration_ms: i64,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct WorkflowRepo {
    db: Pool<Postgres>,
}

impl WorkflowRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn enabled(&self, slug: &str) -> Result<Option<bool>, sqlx::Error> {
        sqlx::query_scalar!("SELECT enabled FROM workflows WHERE slug = $1", slug)
            .fetch_optional(&self.db)
            .await
    }

    pub async fn set_enabled(&self, slug: &str, enabled: bool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO workflows (slug, enabled, updated_at) VALUES ($1, $2, now()) \
             ON CONFLICT (slug) DO UPDATE SET enabled = EXCLUDED.enabled, updated_at = now()",
            slug,
            enabled
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn state_value(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar!("SELECT value FROM state WHERE key = $1", key)
            .fetch_optional(&self.db)
            .await
    }

    pub async fn clear_state_value(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM state WHERE key = $1", key)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    pub async fn set_state_value(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO state (key, value) VALUES ($1, $2) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
            key,
            value
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn append_run(&self, run: NewWorkflowRun<'_>) -> Result<(), sqlx::Error> {
        let NewWorkflowRun {
            slug,
            name,
            event_id,
            outcome,
            dry_run,
            duration_ms,
            error,
        } = run;

        sqlx::query!(
            "INSERT INTO workflow_runs \
             (slug, name, event_id, outcome, dry_run, duration_ms, error) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            slug,
            name,
            event_id,
            outcome,
            dry_run,
            duration_ms,
            error,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn recent_runs(
        &self,
        slug: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WorkflowRunRow>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, slug, name, event_id, outcome, dry_run, duration_ms, error, started_at \
             FROM workflow_runs \
             WHERE ($1::text IS NULL OR slug = $1) \
             ORDER BY started_at DESC \
             LIMIT $2",
            slug,
            limit,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| WorkflowRunRow {
                id: row.id,
                slug: row.slug,
                name: row.name,
                event_id: row.event_id,
                outcome: row.outcome,
                dry_run: row.dry_run,
                duration_ms: row.duration_ms,
                error: row.error,
                started_at: row.started_at,
            })
            .collect())
    }

    pub async fn cooldown_ok(&self, name: &str, cooldown: TimeDelta) -> Result<bool, sqlx::Error> {
        let now = Utc::now();

        let last = sqlx::query!(
            "SELECT last_fired FROM trigger_cooldowns WHERE name = $1",
            name
        )
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = last
            && now - row.last_fired < cooldown
        {
            return Ok(false);
        }

        sqlx::query!(
            "INSERT INTO trigger_cooldowns (name, last_fired) VALUES ($1, $2) \
             ON CONFLICT (name) DO UPDATE SET last_fired = EXCLUDED.last_fired",
            name,
            now
        )
        .execute(&self.db)
        .await?;

        Ok(true)
    }
}
