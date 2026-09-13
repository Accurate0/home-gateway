use async_graphql::{Json, Object};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::actors::system::rpc;
use crate::actors::workflows::{WorkflowWorker, WorkflowWorkerMessage};

use crate::actors::workflows::manager::WorkflowManager;
use crate::auth::scope::{Action, Resource, Scope};
use crate::event_bus::{EventBus, EventBusMessage};
use crate::graphql::guard::ScopeGuard;
use crate::mode::Mode;
use crate::settings::SettingsContainer;
use crate::settings::workflow::scope::callable_inputs;
use crate::variables::Vars;
use crate::variables::input::input_node;

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

    #[graphql(guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Run)))]
    async fn run_workflow(
        &self,
        ctx: &async_graphql::Context<'_>,
        slug: String,
        inputs: Option<Json<BTreeMap<String, serde_json::Value>>>,
    ) -> async_graphql::Result<bool> {
        let settings = ctx.data::<SettingsContainer>()?;
        let definition = settings
            .workflows
            .values()
            .find(|w| w.body().slug == slug)
            .ok_or_else(|| async_graphql::Error::new(format!("unknown workflow slug: {slug}")))?;

        let declared = callable_inputs(definition).map_err(async_graphql::Error::new)?;
        let given = inputs.map(|inputs| inputs.0).unwrap_or_default();
        let input = input_node(&declared, &given).map_err(async_graphql::Error::new)?;

        let message = WorkflowWorkerMessage::Execute {
            event_id: Uuid::new_v4(),
            workflow: definition.body().clone(),
            vars: Vars::default().with("input", input),
            traceparent: crate::tracing_context::inject_current(),
        };

        rpc::cast_factory(WorkflowWorker::NAME, message)
            .map_err(|e| async_graphql::Error::new(format!("error dispatching workflow: {e}")))?;

        Ok(true)
    }
}
