use moka::future::Cache;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use uuid::Uuid;

use crate::mode::Mode;
use crate::repo::WorkflowRepo;
use crate::repo::workflow::NewWorkflowRun;

#[derive(Clone)]
pub struct WorkflowManager {
    repo: WorkflowRepo,
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
    pub fn new(db: Pool<Postgres>) -> Self {
        let enabled_cache = Cache::builder()
            .max_capacity(1024)
            .time_to_live(Duration::from_secs(300))
            .build();

        Self {
            repo: WorkflowRepo::new(db),
            enabled_cache,
        }
    }

    pub async fn enabled(&self, slug: &str, config_default: bool) -> bool {
        let repo = self.repo.clone();
        let slug_owned = slug.to_owned();
        let override_value = self
            .enabled_cache
            .try_get_with(
                slug.to_owned(),
                async move { repo.enabled(&slug_owned).await },
            )
            .await;

        match override_value {
            Ok(value) => value.unwrap_or(config_default),
            Err(err) => {
                tracing::warn!("failed to read workflow override for '{slug}': {err}");
                config_default
            }
        }
    }

    pub async fn mode_active(&self, mode: Mode) -> bool {
        match self.repo.state_value(&mode.state_key()).await {
            Ok(Some(value)) => value == "true",
            Ok(None) => false,
            Err(err) => {
                tracing::warn!("failed to read mode {}: {err}", mode.as_str());
                false
            }
        }
    }

    pub async fn active_modes(&self) -> Vec<Mode> {
        let mut active = Vec::new();
        for mode in Mode::ALL {
            if self.mode_active(*mode).await {
                active.push(*mode);
            }
        }
        active
    }

    /// Set `mode` to `active`, clearing any mutually-exclusive peers when
    /// activating. Returns only the modes whose value actually changed so the
    /// caller can publish a transition event per change.
    pub async fn set_mode(
        &self,
        mode: Mode,
        active: bool,
    ) -> Result<Vec<(Mode, bool)>, sqlx::Error> {
        let mut targets = vec![(mode, active)];
        if active {
            targets.extend(mode.exclusive_peers().map(|peer| (peer, false)));
        }

        let mut transitions = Vec::new();
        for (m, want) in targets {
            if self.mode_active(m).await != want {
                self.write_mode(m, want).await?;
                transitions.push((m, want));
            }
        }
        Ok(transitions)
    }

    async fn write_mode(&self, mode: Mode, active: bool) -> Result<(), sqlx::Error> {
        let value = if active { "true" } else { "false" };

        self.repo.set_state_value(&mode.state_key(), value).await
    }

    pub async fn set_enabled(&self, slug: &str, enabled: bool) -> Result<(), sqlx::Error> {
        self.repo.set_enabled(slug, enabled).await?;

        self.enabled_cache
            .insert(slug.to_owned(), Some(enabled))
            .await;
        Ok(())
    }

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
            })
            .await
        {
            tracing::warn!("failed to record workflow run for '{}': {err}", run.slug);
        }
    }

    pub async fn recent_runs(
        &self,
        slug: Option<&str>,
        limit: i64,
    ) -> Result<Vec<crate::repo::workflow::WorkflowRunRow>, sqlx::Error> {
        self.repo.recent_runs(slug, limit).await
    }

    pub async fn cooldown_ok(
        &self,
        name: &str,
        cooldown: chrono::TimeDelta,
    ) -> Result<bool, sqlx::Error> {
        self.repo.cooldown_ok(name, cooldown).await
    }
}
