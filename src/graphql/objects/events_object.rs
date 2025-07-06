use super::{appliances_object::ApplianceEvent, doors_object::DoorEvent, wifi_object::WifiEvent};
use crate::types::db::{ApplianceStateType, DoorState, UnifiState};
use async_graphql::Object;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use sqlx::{Pool, Postgres};

pub struct EventsObject {
    pub since: DateTime<Utc>,
}

#[Object]
impl EventsObject {
    pub async fn doors(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<DoorEvent>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(sqlx::query!(
            r#"SELECT event_id, id, name, time, state AS "state: DoorState" FROM derived_door_events WHERE time >= $1"#,
            self.since
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| DoorEvent {
            id: r.event_id,
            entity_id: r.id,
            time: r.time,
            state: r.state,
            name: r.name,
        })
        .collect_vec())
    }

    pub async fn wifi(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<WifiEvent>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(sqlx::query!(
            r#"SELECT name, relay_id as id, time, state as "state: UnifiState" FROM unifi_clients WHERE time >= $1"#, self.since
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| WifiEvent {
            time: r.time,
            id: r.id,
            state: r.state,
            name: r.name,
        })
        .collect_vec())
    }

    pub async fn appliances(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<ApplianceEvent>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(sqlx::query!(
            r#"SELECT event_id, name, id, time, state as "state: ApplianceStateType" FROM appliances WHERE time >= $1"#, self.since
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| ApplianceEvent {
            id: r.event_id,
            time: r.time,
            entity_id: r.id,
            state: r.state,
            name: r.name,
        })
        .collect_vec())
    }
}
