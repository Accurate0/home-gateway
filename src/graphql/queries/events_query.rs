use crate::graphql::objects::{environment_object::EnvironmentObject, events_object::EventsObject};
use async_graphql::{InputObject, Object};
use chrono::{DateTime, Utc};

#[derive(InputObject)]
pub struct EventsInput {
    pub since: DateTime<Utc>,
}

#[derive(Default)]
pub struct EventsQuery;

#[Object]
impl EventsQuery {
    async fn events(
        &self,
        _ctx: &async_graphql::Context<'_>,
        input: EventsInput,
    ) -> async_graphql::Result<EventsObject> {
        Ok(EventsObject { since: input.since })
    }

    async fn environment(
        &self,

        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnvironmentObject> {
        Ok(EnvironmentObject {})
    }
}
