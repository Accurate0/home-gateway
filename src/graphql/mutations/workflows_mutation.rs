use async_graphql::Object;
use uuid::Uuid;

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
        let known = settings.workflows.values().any(|w| w.slug == slug);
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
        active: bool,
    ) -> async_graphql::Result<Vec<Mode>> {
        let manager = crate::graphql::require::<WorkflowManager>(ctx, "workflows")?;
        let event_bus = ctx.data::<EventBus>()?;

        let transitions = manager.set_mode(mode, active).await?;
        for (mode, active) in transitions {
            event_bus.publish(EventBusMessage::Mode {
                event_id: Uuid::new_v4(),
                mode,
                active,
            });
        }

        Ok(manager.active_modes().await)
    }

    #[graphql(
        guard = ScopeGuard(Scope::new(Resource::Workflow, Action::Write)),
        deprecation = "use setMode(mode: GUEST, active: ...) instead"
    )]
    async fn set_guest_mode(
        &self,
        ctx: &async_graphql::Context<'_>,
        active: bool,
    ) -> async_graphql::Result<bool> {
        self.set_mode(ctx, Mode::Guest, active).await?;
        Ok(active)
    }
}
