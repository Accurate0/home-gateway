use async_graphql::Object;

use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::mode_status::ModeStatus;
use crate::graphql::objects::workflow_object::{WorkflowRun, WorkflowStatus};
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

        let overrides = manager.enabled_overrides().await;

        let mut statuses = Vec::with_capacity(settings.workflows.len());
        for definition in settings.workflows.values() {
            let workflow = definition.body();
            let enabled = overrides
                .get(&workflow.slug)
                .copied()
                .unwrap_or(workflow.enabled);
            statuses.push(WorkflowStatus {
                id: workflow.slug.clone(),
                slug: workflow.slug.clone(),
                name: workflow.name.clone(),
                group: workflow.group.clone().unwrap_or_else(|| "Other".to_owned()),
                tags: workflow.tags.clone(),
                enabled,
                config_enabled: workflow.enabled,
                dry_run: workflow.dry_run,
                reusable: definition.triggered().is_none(),
                modes: definition
                    .triggered()
                    .map(|triggered| triggered.modes.clone())
                    .unwrap_or_default(),
            });
        }
        statuses.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(statuses)
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Read)))]
    async fn mode(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<ModeStatus> {
        let manager = crate::graphql::require::<WorkflowManager>(ctx, "workflows")?;

        Ok(ModeStatus {
            active: manager.current_mode().await,
        })
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Read)))]
    async fn workflow_runs(
        &self,
        ctx: &async_graphql::Context<'_>,
        slug: Option<String>,
        event_id: Option<uuid::Uuid>,
        limit: Option<i64>,
    ) -> async_graphql::Result<Vec<WorkflowRun>> {
        let workflows = crate::graphql::require::<WorkflowManager>(ctx, "workflows")?;
        let limit = limit.unwrap_or(50).clamp(1, 500);

        let rows = workflows
            .recent_runs(slug.as_deref(), event_id, limit)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| WorkflowRun {
                id: async_graphql::ID(r.id.to_string()),
                run_id: r.id,
                trigger: r.trigger.map(async_graphql::Json),
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
