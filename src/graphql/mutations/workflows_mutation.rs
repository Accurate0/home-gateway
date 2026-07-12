use async_graphql::Object;

use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::settings::SettingsContainer;

#[derive(Default)]
pub struct WorkflowsMutation;

#[Object]
impl WorkflowsMutation {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_WORKFLOW_WRITE))]
    async fn set_workflow_enabled(
        &self,
        ctx: &async_graphql::Context<'_>,
        slug: String,
        enabled: bool,
    ) -> async_graphql::Result<bool> {
        let settings = ctx.data::<SettingsContainer>()?;
        let known = settings.workflows.values().any(|w| w.slug == slug);
        if !known {
            return Err(async_graphql::Error::new(format!(
                "unknown workflow slug: {slug}"
            )));
        }

        let manager = ctx.data::<WorkflowManager>()?;
        manager.set_enabled(&slug, enabled).await?;
        Ok(enabled)
    }
}
