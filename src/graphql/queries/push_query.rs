use std::collections::HashMap;

use async_graphql::Object;
use uuid::Uuid;

use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::push_notification_interaction_object::PushNotificationInteractionObject;
use crate::graphql::objects::push_notification_object::PushNotificationObject;
use crate::repo::RepoRegistry;
use crate::repo::notification_interaction::NotificationInteraction;

#[derive(Default)]
pub struct PushQuery;

#[Object]
impl PushQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Push, Action::Read)))]
    async fn push_notifications(
        &self,
        ctx: &async_graphql::Context<'_>,
        limit: Option<i64>,
    ) -> async_graphql::Result<Vec<PushNotificationObject>> {
        let repos = ctx.data::<RepoRegistry>()?;
        let limit = limit.unwrap_or(50).clamp(1, 500);

        let rows = repos.push().recent_notifications(limit).await?;
        let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let interactions = repos.push().interactions(&ids).await?;

        let mut by_notification: HashMap<Uuid, Vec<PushNotificationInteractionObject>> =
            HashMap::new();

        for interaction in interactions {
            let Some(kind) = NotificationInteraction::parse(&interaction.kind) else {
                tracing::warn!(
                    "skipping unknown interaction kind '{}' on notification {}",
                    interaction.kind,
                    interaction.notification_id
                );
                continue;
            };

            by_notification
                .entry(interaction.notification_id)
                .or_default()
                .push(PushNotificationInteractionObject {
                    kind: kind.into(),
                    at: interaction.at,
                });
        }

        Ok(rows
            .into_iter()
            .map(|row| PushNotificationObject {
                interactions: by_notification.remove(&row.id).unwrap_or_default(),
                id: row.id,
                tag: row.tag,
                title: row.title,
                body: row.body,
                category: row.category,
                send_count: row.send_count,
                next_reminder_at: row.next_reminder_at,
                acknowledged_at: row.acknowledged_at,
                created_at: row.created_at,
            })
            .collect())
    }
}
