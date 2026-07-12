use moka::future::Cache;
use sqlx::{Pool, Postgres};
use std::time::Duration;

#[derive(Clone)]
pub struct WorkflowManager {
    db: Pool<Postgres>,
    enabled_cache: Cache<String, Option<bool>>,
}

impl WorkflowManager {
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
}
