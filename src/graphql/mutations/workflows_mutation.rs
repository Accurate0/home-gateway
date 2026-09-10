use async_graphql::Object;
use std::collections::HashMap;
use uuid::Uuid;

use crate::actors::system::rpc;
use crate::actors::workflows::{WorkflowWorker, WorkflowWorkerMessage};

use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::scope::{Action, Resource, Scope};
use crate::event_bus::{EventBus, EventBusMessage};
use crate::graphql::guard::ScopeGuard;
use crate::mode::Mode;
use crate::settings::SettingsContainer;

#[derive(Default)]
pub struct WorkflowsMutation;

#[Object]
impl WorkflowsMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Write)))]
    async fn set_workflow_enabled(
        &self,
        ctx: &async_graphql::Context<'_>,
        slug: String,
        enabled: bool,
    ) -> async_graphql::Result<bool> {
        let settings = ctx.data::<SettingsContainer>()?;
        let known = settings.workflows.values().any(|w| w.body().slug == slug);
        if !known {
            return Err(async_graphql::Error::new(format!(
                "unknown workflow slug: {slug}"
            )));
        }

        let manager = crate::graphql::require::<WorkflowManager>(ctx, "workflows")?;
        manager.set_enabled(&slug, enabled).await?;
        Ok(enabled)
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Write)))]
    async fn set_mode(
        &self,
        ctx: &async_graphql::Context<'_>,
        mode: Mode,
    ) -> async_graphql::Result<Mode> {
        let manager = crate::graphql::require::<WorkflowManager>(ctx, "workflows")?;
        let event_bus = ctx.data::<EventBus>()?;

        if let Some(previous) = manager.set_mode(mode).await? {
            event_bus.publish(EventBusMessage::Mode {
                event_id: Uuid::new_v4(),
                mode,
                previous,
            });
        }

        Ok(mode)
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Write)))]
    async fn run_workflow(
        &self,
        ctx: &async_graphql::Context<'_>,
        slug: String,
    ) -> async_graphql::Result<bool> {
        let settings = ctx.data::<SettingsContainer>()?;
        let workflow = settings
            .workflows
            .values()
            .map(crate::settings::WorkflowDefinition::body)
            .find(|w| w.slug == slug)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown workflow slug: {slug}")))?
            .clone();

        let message = WorkflowWorkerMessage::Execute {
            event_id: Uuid::new_v4(),
            workflow,
            vars: HashMap::new(),
            traceparent: crate::tracing_context::inject_current(),
        };

        rpc::cast_factory(WorkflowWorker::NAME, message)
            .map_err(|e| async_graphql::Error::new(format!("error dispatching workflow: {e}")))?;

        Ok(true)
    }
}
