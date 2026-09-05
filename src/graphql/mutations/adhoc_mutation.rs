use async_graphql::Object;

use crate::actors::system::adhoc::{AdhocTaskActor, AdhocTaskActorMessage};
use crate::actors::system::rpc;
use crate::adhoc::cron_registry;
use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;

#[derive(Default)]
pub struct AdhocMutation;

#[Object]
impl AdhocMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::AdhocTask, Action::Write)))]
    async fn run_pending_adhoc_tasks(&self) -> async_graphql::Result<bool> {
        rpc::cast(AdhocTaskActor::NAME, AdhocTaskActorMessage::RunPending)?;

        Ok(true)
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::AdhocTask, Action::Write)))]
    async fn run_adhoc_cron_task(&self, name: String) -> async_graphql::Result<bool> {
        let Some(task) = cron_registry().into_iter().find(|task| task.name() == name) else {
            return Err(async_graphql::Error::new(format!(
                "unknown adhoc cron task: {name}"
            )));
        };

        rpc::cast(
            AdhocTaskActor::NAME,
            AdhocTaskActorMessage::RunCron { name: task.name() },
        )?;

        Ok(true)
    }
}
