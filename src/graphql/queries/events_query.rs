use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::{
    environment_object::EnvironmentObject, events_object::EventsObject,
    home_assistant_object::HomeAssistantEvent,
};
use async_graphql::{InputObject, Object};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use sqlx::{Pool, Postgres};

#[derive(InputObject)]
pub struct EventsInput {
    pub since: DateTime<Utc>,
}

#[derive(Default)]
pub struct EventsQuery;

#[Object]
impl EventsQuery {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_EVENTS_READ))]
    async fn events(
        &self,
        _ctx: &async_graphql::Context<'_>,
        input: EventsInput,
    ) -> async_graphql::Result<EventsObject> {
        Ok(EventsObject { since: input.since })
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_EVENTS_READ))]
    async fn home_assistant_entities(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<HomeAssistantEvent>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(sqlx::query!(
            r#"SELECT event_id, entity_id, state, updated_at FROM latest_home_assistant_state ORDER BY updated_at DESC"#
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| HomeAssistantEvent {
            event_id: r.event_id,
            entity_id: r.entity_id,
            state: r.state,
            time: r.updated_at,
        })
        .collect_vec())
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_ENTITY_READ))]
    async fn environment(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnvironmentObject> {
        Ok(EnvironmentObject {})
    }
}
