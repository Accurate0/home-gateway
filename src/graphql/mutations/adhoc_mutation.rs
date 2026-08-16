use async_graphql::Object;

use crate::actors::system::adhoc::{AdhocTaskActor, AdhocTaskActorMessage};
use crate::adhoc::cron_registry;
use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;

#[derive(Default)]
pub struct AdhocMutation;

#[Object]
impl AdhocMutation {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_ADHOC_TASK_EXECUTE))]
    async fn run_pending_adhoc_tasks(&self) -> async_graphql::Result<bool> {
        let Some(actor) = ractor::registry::where_is(AdhocTaskActor::NAME) else {
            return Err(async_graphql::Error::new("adhoc task actor unavailable"));
        };

        actor.send_message(AdhocTaskActorMessage::RunPending)?;

        Ok(true)
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_ADHOC_TASK_EXECUTE))]
    async fn run_adhoc_cron_task(&self, name: String) -> async_graphql::Result<bool> {
        let Some(task) = cron_registry().into_iter().find(|task| task.name() == name) else {
            return Err(async_graphql::Error::new(format!(
                "unknown adhoc cron task: {name}"
            )));
        };

        let Some(actor) = ractor::registry::where_is(AdhocTaskActor::NAME) else {
            return Err(async_graphql::Error::new("adhoc task actor unavailable"));
        };

        actor.send_message(AdhocTaskActorMessage::RunCron { name: task.name() })?;

        Ok(true)
    }
}
