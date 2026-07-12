use async_graphql::Object;
use sqlx::{Pool, Postgres};

use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::workflow_object::{WorkflowRun, WorkflowStatus};
use crate::mode::Mode;
use crate::settings::SettingsContainer;

#[derive(Default)]
pub struct WorkflowsQuery;

#[Object]
impl WorkflowsQuery {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_WORKFLOW_READ))]
    async fn workflows(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<WorkflowStatus>> {
        let settings = ctx.data::<SettingsContainer>()?;
        let manager = ctx.data::<WorkflowManager>()?;

        let mut statuses = Vec::with_capacity(settings.workflows.len());
        for workflow in settings.workflows.values() {
            let enabled = manager.enabled(&workflow.slug, workflow.enabled).await;
            statuses.push(WorkflowStatus {
                id: workflow.slug.clone(),
                slug: workflow.slug.clone(),
                name: workflow.name.clone(),
                group: workflow.group.clone().unwrap_or_else(|| "Other".to_owned()),
                enabled,
                config_enabled: workflow.enabled,
                dry_run: workflow.dry_run,
                reusable: workflow.on().is_none(),
            });
        }
        statuses.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(statuses)
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_WORKFLOW_READ))]
    async fn active_modes(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<Mode>> {
        let manager = ctx.data::<WorkflowManager>()?;
        Ok(manager.active_modes().await)
    }

    #[graphql(
        guard = ScopeGuard(required::GRAPHQL_WORKFLOW_READ),
        deprecation = "use activeModes / mode(GUEST) instead"
    )]
    async fn guest_mode(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let manager = ctx.data::<WorkflowManager>()?;
        Ok(manager.mode_active(Mode::Guest).await)
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_WORKFLOW_READ))]
    async fn workflow_runs(
        &self,
        ctx: &async_graphql::Context<'_>,
        slug: Option<String>,
        limit: Option<i64>,
    ) -> async_graphql::Result<Vec<WorkflowRun>> {
        let db = ctx.data::<Pool<Postgres>>()?;
        let limit = limit.unwrap_or(50).clamp(1, 500);

        let rows = sqlx::query!(
            "SELECT id, slug, name, event_id, outcome, dry_run, duration_ms, error, started_at \
             FROM workflow_runs \
             WHERE ($1::text IS NULL OR slug = $1) \
             ORDER BY started_at DESC \
             LIMIT $2",
            slug,
            limit,
        )
        .fetch_all(db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| WorkflowRun {
                id: async_graphql::ID(r.id.to_string()),
                slug: r.slug,
                name: r.name,
                event_id: r.event_id.to_string(),
                outcome: r.outcome,
                dry_run: r.dry_run,
                duration_ms: r.duration_ms,
                error: r.error,
                started_at: r.started_at,
            })
            .collect())
    }
}
