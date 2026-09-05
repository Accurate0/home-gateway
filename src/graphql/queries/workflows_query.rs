use async_graphql::Object;

use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::workflow_object::{WorkflowRun, WorkflowStatus};
use crate::mode::Mode;
use crate::settings::SettingsContainer;

#[derive(Default)]
pub struct WorkflowsQuery;

#[Object]
impl WorkflowsQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Read)))]
    async fn workflows(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<WorkflowStatus>> {
        let settings = ctx.data::<SettingsContainer>()?;
        let manager = crate::graphql::require::<WorkflowManager>(ctx, "workflows")?;

        let mut statuses = Vec::with_capacity(settings.workflows.len());
        for workflow in settings.workflows.values() {
            let enabled = manager.enabled(&workflow.slug, workflow.enabled).await;
            statuses.push(WorkflowStatus {
                id: workflow.slug.clone(),
                slug: workflow.slug.clone(),
                name: workflow.name.clone(),
                group: workflow.group.clone().unwrap_or_else(|| "Other".to_owned()),
                tags: workflow.tags.clone(),
                enabled,
                config_enabled: workflow.enabled,
                dry_run: workflow.dry_run,
                reusable: workflow.on().is_none(),
            });
        }
        statuses.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(statuses)
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Read)))]
    async fn active_modes(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<Mode>> {
        let manager = crate::graphql::require::<WorkflowManager>(ctx, "workflows")?;
        Ok(manager.active_modes().await)
    }

    #[graphql(
        guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Read)),
        deprecation = "use activeModes / mode(GUEST) instead"
    )]
    async fn guest_mode(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let manager = crate::graphql::require::<WorkflowManager>(ctx, "workflows")?;
        Ok(manager.mode_active(Mode::Guest).await)
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Read)))]
    async fn workflow_runs(
        &self,
        ctx: &async_graphql::Context<'_>,
        slug: Option<String>,
        limit: Option<i64>,
    ) -> async_graphql::Result<Vec<WorkflowRun>> {
        let workflows = crate::graphql::require::<WorkflowManager>(ctx, "workflows")?;
        let limit = limit.unwrap_or(50).clamp(1, 500);

        let rows = workflows.recent_runs(slug.as_deref(), limit).await?;

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
