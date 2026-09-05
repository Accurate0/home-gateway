use crate::repo::RepoRegistry;
use async_graphql::{ComplexObject, ID, Object, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use uuid::Uuid;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct HomeAssistantEvent {
    pub event_id: Uuid,
    pub entity_id: String,
    pub state: String,
    pub time: DateTime<Utc>,
}

#[ComplexObject]
impl HomeAssistantEvent {
    async fn id(&self) -> ID {
        ID(self.event_id.to_string())
    }
}

pub struct HomeAssistantObject {}

#[Object]
impl HomeAssistantObject {
    async fn entities(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<HomeAssistantEvent>> {
        let repos = ctx.data::<RepoRegistry>()?;

        Ok(repos
            .home_assistant()
            .latest_all()
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
}
