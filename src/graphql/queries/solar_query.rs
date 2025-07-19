use crate::graphql::objects::solar_object::SolarObject;
use async_graphql::{InputObject, Object};
use chrono::{DateTime, Utc};

#[derive(InputObject)]
pub struct SolarInput {
    pub since: DateTime<Utc>,
}

#[derive(Default)]
pub struct SolarQuery;

#[Object]
impl SolarQuery {
    async fn solar(
        &self,
        _ctx: &async_graphql::Context<'_>,
        input: SolarInput,
    ) -> async_graphql::Result<SolarObject> {
        Ok(SolarObject { since: input.since })
    }
}
