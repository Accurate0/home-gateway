use async_graphql::Object;
use uuid::Uuid;

use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::notification_interaction_kind::NotificationInteractionKind;
use crate::repo::RepoRegistry;
use crate::repo::notification_interaction::NotificationInteraction;

#[derive(Default)]
pub struct PushMutation;

#[Object]
impl PushMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Push, Action::Write)))]
    async fn record_notification_interaction(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: Uuid,
        kind: NotificationInteractionKind,
    ) -> async_graphql::Result<bool> {
        let repos = ctx.data::<RepoRegistry>()?;
        let kind = NotificationInteraction::from(kind);

        let recorded = repos.push().record_interaction(id, kind).await?;

        if !recorded {
            return Err(async_graphql::Error::new(format!(
                "unknown notification: {id}"
            )));
        }

        tracing::info!("notification {id} {}", kind.as_str());

        Ok(true)
    }
}
