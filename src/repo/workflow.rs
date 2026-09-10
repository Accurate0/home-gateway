use chrono::{DateTime, TimeDelta, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use super::timer_kind::TimerKind;

pub struct NewWorkflowRun<'a> {
    pub slug: &'a str,
    pub name: &'a str,
    pub event_id: Uuid,
    pub outcome: &'a str,
    pub dry_run: bool,
    pub duration_ms: i64,
    pub error: Option<&'a str>,
}

pub struct PendingTimerRow {
    pub id: Uuid,
    pub workflow: String,
    pub timer_kind: String,
    pub subject_kind: String,
    pub subject_entity: String,
    pub event_id: Uuid,
    pub vars: serde_json::Value,
    pub fire_at: DateTime<Utc>,
}

pub struct NewPendingTimer<'a> {
    pub workflow: &'a str,
    pub kind: TimerKind,
    pub subject_kind: &'a str,
    pub subject_entity: &'a str,
    pub event_id: Uuid,
    pub vars: serde_json::Value,
    pub fire_at: DateTime<Utc>,
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

    #[tracing::instrument(skip_all, name = "db.workflow.enabled", err)]
    pub async fn enabled(&self, slug: &str) -> Result<Option<bool>, sqlx::Error> {
        sqlx::query_scalar!("SELECT enabled FROM workflows WHERE slug = $1", slug)
            .fetch_optional(&self.db)
            .await
    }

    #[tracing::instrument(skip_all, name = "db.workflow.set_enabled", err)]
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

    #[tracing::instrument(skip_all, name = "db.workflow.state_value", err)]
    pub async fn state_value(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar!("SELECT value FROM state WHERE key = $1", key)
            .fetch_optional(&self.db)
            .await
    }

    #[tracing::instrument(skip_all, name = "db.workflow.clear_state_value", err)]
    pub async fn clear_state_value(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM state WHERE key = $1", key)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.workflow.set_state_value", err)]
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

    #[tracing::instrument(skip_all, name = "db.workflow.append_run", err)]
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

    #[tracing::instrument(skip_all, name = "db.workflow.recent_runs", err)]
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

    #[tracing::instrument(skip_all, name = "db.workflow.cooldown_ok", err)]
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

    #[tracing::instrument(skip_all, name = "db.workflow.arm_timer", err)]
    pub async fn arm_timer(
        &self,
        timer: NewPendingTimer<'_>,
    ) -> Result<Option<PendingTimerRow>, sqlx::Error> {
        let NewPendingTimer {
            workflow,
            kind,
            subject_kind,
            subject_entity,
            event_id,
            vars,
            fire_at,
        } = timer;

        match kind {
            TimerKind::Hold => {
                sqlx::query_as!(
                    PendingTimerRow,
                    "INSERT INTO workflow_pending_timer \
                     (workflow, timer_kind, subject_kind, subject_entity, event_id, vars, fire_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7) \
                     ON CONFLICT (workflow, timer_kind) DO NOTHING \
                     RETURNING id, workflow, timer_kind, subject_kind, subject_entity, event_id, vars, fire_at",
                    workflow,
                    kind.as_str(),
                    subject_kind,
                    subject_entity,
                    event_id,
                    vars,
                    fire_at,
                )
                .fetch_optional(&self.db)
                .await
            }
            TimerKind::Delay => {
                sqlx::query_as!(
                    PendingTimerRow,
                    "INSERT INTO workflow_pending_timer \
                     (workflow, timer_kind, subject_kind, subject_entity, event_id, vars, fire_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7) \
                     ON CONFLICT (workflow, timer_kind) DO UPDATE SET \
                     id = gen_random_uuid(), subject_kind = EXCLUDED.subject_kind, \
                     subject_entity = EXCLUDED.subject_entity, event_id = EXCLUDED.event_id, \
                     vars = EXCLUDED.vars, armed_at = now(), fire_at = EXCLUDED.fire_at \
                     RETURNING id, workflow, timer_kind, subject_kind, subject_entity, event_id, vars, fire_at",
                    workflow,
                    kind.as_str(),
                    subject_kind,
                    subject_entity,
                    event_id,
                    vars,
                    fire_at,
                )
                .fetch_optional(&self.db)
                .await
            }
        }
    }

    #[tracing::instrument(skip_all, name = "db.workflow.cancel_timer", err)]
    pub async fn cancel_timer(
        &self,
        workflow: &str,
        kind: TimerKind,
        subject_kind: &str,
        subject_entity: &str,
    ) -> Result<bool, sqlx::Error> {
        let deleted = sqlx::query!(
            "DELETE FROM workflow_pending_timer \
             WHERE workflow = $1 AND timer_kind = $2 AND subject_kind = $3 AND subject_entity = $4",
            workflow,
            kind.as_str(),
            subject_kind,
            subject_entity,
        )
        .execute(&self.db)
        .await?
        .rows_affected();

        Ok(deleted > 0)
    }

    #[tracing::instrument(skip_all, name = "db.workflow.cancel_timers_for_subject", err)]
    pub async fn cancel_timers_for_subject(
        &self,
        kind: TimerKind,
        subject_kind: &str,
        subject_entity: &str,
    ) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar!(
            "DELETE FROM workflow_pending_timer \
             WHERE timer_kind = $1 AND subject_kind = $2 AND subject_entity = $3 \
             RETURNING workflow",
            kind.as_str(),
            subject_kind,
            subject_entity,
        )
        .fetch_all(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.workflow.take_timer", err)]
    pub async fn take_timer(&self, id: Uuid) -> Result<Option<PendingTimerRow>, sqlx::Error> {
        sqlx::query_as!(
            PendingTimerRow,
            "DELETE FROM workflow_pending_timer WHERE id = $1 \
             RETURNING id, workflow, timer_kind, subject_kind, subject_entity, event_id, vars, fire_at",
            id,
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.workflow.pending_timers", err)]
    pub async fn pending_timers(&self) -> Result<Vec<PendingTimerRow>, sqlx::Error> {
        sqlx::query_as!(
            PendingTimerRow,
            "SELECT id, workflow, timer_kind, subject_kind, subject_entity, event_id, vars, fire_at \
             FROM workflow_pending_timer ORDER BY fire_at"
        )
        .fetch_all(&self.db)
        .await
    }
}
