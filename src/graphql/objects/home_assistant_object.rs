use async_graphql::{ComplexObject, ID, Object, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use sqlx::{Pool, Postgres};
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
}
