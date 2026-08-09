use crate::integrations::solar::queries;
use crate::integrations::solar::types::{GenerationHistory, SolarCurrentResponse};
use async_graphql::{InputObject, Object};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

pub struct SolarObject {}

#[derive(InputObject)]
pub struct SolarHistoryInput {
    pub since: DateTime<Utc>,
}

#[Object]
impl SolarObject {
    pub async fn current(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<SolarCurrentResponse> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(queries::current(db).await?)
    }

    pub async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        input: SolarHistoryInput,
    ) -> async_graphql::Result<Vec<GenerationHistory>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(queries::history_since(db, input.since).await?)
    }
}
