use async_graphql::Object;
use uuid::Uuid;

use crate::actors::system::push::types::PushActionKind;
use crate::actors::system::push::{PushActor, PushMessage, PushNotification};
use crate::actors::system::rpc;
use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::notification_interaction_kind::NotificationInteractionKind;
use crate::graphql::objects::send_push_notification_input::SendPushNotificationInput;
use crate::repo::RepoRegistry;
use crate::repo::notification_interaction::NotificationInteraction;
use crate::settings::{NotifyAcknowledge, SettingsContainer, validate_acknowledge};

#[derive(Default)]
pub struct PushMutation;

#[Object]
impl PushMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Push, Action::Write)))]
    async fn send_push_notification(
        &self,
        ctx: &async_graphql::Context<'_>,
        input: SendPushNotificationInput,
    ) -> async_graphql::Result<bool> {
        let settings = ctx.data::<SettingsContainer>()?;

        let actions = input
            .actions
            .into_iter()
            .map(|action| action.into_push_action(settings))
            .collect::<Result<Vec<_>, String>>()
            .map_err(async_graphql::Error::new)?;

        let acknowledge = input
            .acknowledge
            .map(NotifyAcknowledge::try_from)
            .transpose()
            .map_err(async_graphql::Error::new)?;

        let has_acknowledge_action = actions
            .iter()
            .any(|action| matches!(action.kind, PushActionKind::Acknowledge));

        validate_acknowledge(acknowledge.is_some(), has_acknowledge_action)
            .map_err(async_graphql::Error::new)?;

        let tag = input.tag.unwrap_or_else(|| format!("push:{}", input.title));

        tracing::info!("sending push notification \"{}\" via graphql", input.body);

        let message = PushMessage::Send(PushNotification {
            title: input.title,
            body: input.body,
            category: input.category,
            tag,
            actions,
            acknowledge,
        });

        rpc::cast(PushActor::NAME, message).map_err(|e| {
            async_graphql::Error::new(format!("error sending push notification: {e}"))
        })?;

        Ok(true)
    }

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
