use async_graphql::Object;

use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::workflow_object::WorkflowStatus;
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
                group: workflow
                    .group
                    .clone()
                    .unwrap_or_else(|| "Other".to_owned()),
                enabled,
                config_enabled: workflow.enabled,
                dry_run: workflow.dry_run,
                reusable: workflow.on().is_none(),
            });
        }
        statuses.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(statuses)
    }
}
