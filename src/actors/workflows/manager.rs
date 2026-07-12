use moka::future::Cache;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct WorkflowManager {
    db: Pool<Postgres>,
    enabled_cache: Cache<String, Option<bool>>,
}

pub struct WorkflowRun {
    pub slug: String,
    pub name: String,
    pub event_id: Uuid,
    pub outcome: String,
    pub dry_run: bool,
    pub duration: Duration,
    pub error: Option<String>,
}

impl WorkflowManager {
    const GUEST_MODE_KEY: &str = "mode:guest";

    pub fn new(db: Pool<Postgres>) -> Self {
        let enabled_cache = Cache::builder()
            .max_capacity(1024)
            .time_to_live(Duration::from_secs(300))
            .build();
        Self { db, enabled_cache }
    }

    pub async fn enabled(&self, slug: &str, config_default: bool) -> bool {
        let db = self.db.clone();
        let slug_owned = slug.to_owned();
        let override_value = self
            .enabled_cache
            .try_get_with(slug.to_owned(), async move {
                sqlx::query_scalar!("SELECT enabled FROM workflows WHERE slug = $1", slug_owned)
                    .fetch_optional(&db)
                    .await
            })
            .await;

        match override_value {
            Ok(value) => value.unwrap_or(config_default),
            Err(err) => {
                tracing::warn!("failed to read workflow override for '{slug}': {err}");
                config_default
            }
        }
    }

    pub async fn guest_mode(&self) -> bool {
        let row = sqlx::query_scalar!(
            "SELECT value FROM state WHERE key = $1",
            Self::GUEST_MODE_KEY
        )
        .fetch_optional(&self.db)
        .await;

        match row {
            Ok(Some(value)) => value == "true",
            Ok(None) => false,
            Err(err) => {
                tracing::warn!("failed to read guest mode state: {err}");
                false
            }
        }
    }

    pub async fn set_guest_mode(&self, active: bool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO state (key, value) VALUES ($1, $2) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
            Self::GUEST_MODE_KEY,
            if active { "true" } else { "false" }
        )
        .execute(&self.db)
        .await?;
        Ok(())
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

        self.enabled_cache
            .insert(slug.to_owned(), Some(enabled))
            .await;
        Ok(())
    }

    pub async fn record_run(&self, run: WorkflowRun) {
        let duration_ms = i64::try_from(run.duration.as_millis()).unwrap_or(i64::MAX);
        if let Err(err) = sqlx::query!(
            "INSERT INTO workflow_runs \
             (slug, name, event_id, outcome, dry_run, duration_ms, error) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            run.slug,
            run.name,
            run.event_id,
            run.outcome,
            run.dry_run,
            duration_ms,
            run.error,
        )
        .execute(&self.db)
        .await
        {
            tracing::warn!("failed to record workflow run for '{}': {err}", run.slug);
        }
    }
}
