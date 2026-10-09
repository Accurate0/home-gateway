use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::cache::MemoryCache;
use crate::repo::WorkflowRepo;
use crate::repo::workflow::NewWorkflowRun;
use crate::settings::CacheSettings;
use crate::settings::workflow::WorkflowSettings;
use crate::workflows::mode::Mode;
use crate::workflows::trace::StepTrace;

#[derive(Clone)]
pub struct WorkflowManager {
    repo: WorkflowRepo,
    enabled_cache: MemoryCache<String, Option<bool>>,
    cooldown_cache: MemoryCache<String, DateTime<Utc>>,
    mode_cache: MemoryCache<String, Mode>,
}

pub struct WorkflowRun {
    pub slug: String,
    pub name: String,
    pub event_id: Uuid,
    pub outcome: String,
    pub dry_run: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub trigger: Option<serde_json::Value>,
    pub steps: Vec<StepTrace>,
}

impl WorkflowManager {
    pub fn new(db: Pool<Postgres>, settings: &WorkflowSettings) -> Self {
        Self {
            repo: WorkflowRepo::new(db),
            enabled_cache: build_cache("workflow_enabled", &settings.enabled_cache),
            cooldown_cache: build_cache("workflow_cooldown", &settings.cooldown_cache),
            mode_cache: build_cache("workflow_mode", &settings.mode_cache),
        }
    }

    #[tracing::instrument(
        name = "workflow.enabled",
        skip_all,
        fields(slug = %slug, cached = tracing::field::Empty, enabled = tracing::field::Empty)
    )]
    pub async fn enabled(&self, slug: &str, config_default: bool) -> bool {
        let repo = self.repo.clone();
        let slug_owned = slug.to_owned();
        let override_value = self
            .enabled_cache
            .or_try_insert_with(
                slug.to_owned(),
                async move { repo.enabled(&slug_owned).await },
            )
            .await;

        let span = tracing::Span::current();

        let enabled = match override_value {
            Ok(entry) => {
                span.record("cached", !entry.is_fresh());

                entry.into_value().unwrap_or(config_default)
            }
            Err(err) => {
                tracing::warn!("failed to read workflow override for '{slug}': {err}");
                config_default
            }
        };

        span.record("enabled", enabled);

        enabled
    }

    #[tracing::instrument(name = "workflow.enabled_overrides", skip_all)]
    pub async fn enabled_overrides(&self) -> HashMap<String, bool> {
        match self.repo.enabled_overrides().await {
            Ok(rows) => {
                for (slug, enabled) in &rows {
                    self.enabled_cache
                        .insert(slug.clone(), Some(*enabled))
                        .await;
                }

                rows.into_iter().collect()
            }
            Err(err) => {
                tracing::warn!("failed to read workflow overrides: {err}");

                HashMap::new()
            }
        }
    }

    #[tracing::instrument(name = "workflow.current_mode", skip_all)]
    pub async fn current_mode(&self) -> Mode {
        if let Some(mode) = self.mode_cache.get(Mode::STATE_KEY).await {
            return mode;
        }

        let mode = match self.repo.state_value(Mode::STATE_KEY).await {
            Ok(Some(value)) => Mode::parse(&value).unwrap_or_else(|| {
                tracing::warn!(
                    "unknown stored mode `{value}`, falling back to {}",
                    Mode::default().as_str()
                );

                Mode::default()
            }),
            Ok(None) => Mode::default(),
            Err(err) => {
                tracing::warn!("failed to read the current mode: {err}");

                return Mode::default();
            }
        };

        self.mode_cache
            .insert(Mode::STATE_KEY.to_owned(), mode)
            .await;

        mode
    }

    pub async fn any_mode_active(&self, modes: &[Mode]) -> bool {
        modes.is_empty() || modes.contains(&self.current_mode().await)
    }

    #[tracing::instrument(name = "workflow.set_mode", skip_all, err)]
    pub async fn set_mode(&self, mode: Mode) -> Result<Option<Mode>, sqlx::Error> {
        let previous = self.current_mode().await;

        if previous == mode {
            return Ok(None);
        }

        self.repo
            .set_state_value(Mode::STATE_KEY, mode.as_str())
            .await?;

        self.mode_cache
            .insert(Mode::STATE_KEY.to_owned(), mode)
            .await;

        Ok(Some(previous))
    }

    #[tracing::instrument(name = "workflow.set_enabled", skip_all, fields(slug = %slug, enabled), err)]
    pub async fn set_enabled(&self, slug: &str, enabled: bool) -> Result<(), sqlx::Error> {
        self.repo.set_enabled(slug, enabled).await?;

        self.enabled_cache
            .insert(slug.to_owned(), Some(enabled))
            .await;
        Ok(())
    }

    #[tracing::instrument(name = "workflow.record_run", skip_all, fields(slug = %run.slug))]
    pub async fn record_run(&self, run: WorkflowRun) {
        let duration_ms = i64::try_from(run.duration.as_millis()).unwrap_or(i64::MAX);

        if let Err(err) = self
            .repo
            .append_run(NewWorkflowRun {
                slug: &run.slug,
                name: &run.name,
                event_id: run.event_id,
                outcome: &run.outcome,
                dry_run: run.dry_run,
                duration_ms,
                error: run.error.as_deref(),
                trigger: run.trigger,
                steps: run.steps,
            })
            .await
        {
            tracing::warn!("failed to record workflow run for '{}': {err}", run.slug);
        }
    }

    #[tracing::instrument(name = "workflow.recent_runs", skip_all, err)]
    pub async fn recent_runs(
        &self,
        slug: Option<&str>,
        event_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<crate::repo::workflow::WorkflowRunRow>, sqlx::Error> {
        self.repo.recent_runs(slug, event_id, limit).await
    }

    #[tracing::instrument(name = "workflow.cooldown_ok", skip_all, fields(name = %name), err)]
    pub async fn cooldown_ok(
        &self,
        name: &str,
        cooldown: chrono::TimeDelta,
    ) -> Result<bool, sqlx::Error> {
        let now = Utc::now();

        if let Some(last_fired) = self.cooldown_last_fired(name).await?
            && now - last_fired < cooldown
        {
            return Ok(false);
        }

        self.repo.record_cooldown(name, now).await?;
        self.cooldown_cache.insert(name.to_owned(), now).await;

        Ok(true)
    }

    #[tracing::instrument(name = "workflow.cooldown_active", skip_all, fields(name = %name), err)]
    pub async fn cooldown_active(
        &self,
        name: &str,
        cooldown: chrono::TimeDelta,
    ) -> Result<bool, sqlx::Error> {
        let last_fired = self.cooldown_last_fired(name).await?;

        Ok(last_fired.is_some_and(|last_fired| Utc::now() - last_fired < cooldown))
    }

    async fn cooldown_last_fired(&self, name: &str) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        if let Some(last_fired) = self.cooldown_cache.get(name).await {
            return Ok(Some(last_fired));
        }

        let last_fired = self.repo.cooldown_last_fired(name).await?;

        if let Some(last_fired) = last_fired {
            self.cooldown_cache
                .insert(name.to_owned(), last_fired)
                .await;
        }

        Ok(last_fired)
    }
}

fn build_cache<V>(name: &'static str, settings: &CacheSettings) -> MemoryCache<String, V>
where
    V: Clone + Send + Sync + 'static,
{
    MemoryCache::builder(name)
        .configure(|cache| {
            cache
                .max_capacity(settings.capacity)
                .time_to_live(settings.ttl())
        })
        .build()
}
