use super::{appliances_object::ApplianceEvent, doors_object::DoorEvent, wifi_object::WifiEvent};
use crate::types::db::{ApplianceStateType, UnifiState};
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
            "SELECT name, time, contact FROM door_sensor WHERE time >= $1",
            self.since
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| DoorEvent {
            time: r.time,
            contact: r.contact,
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
            r#"SELECT name, id, time, state as "state: UnifiState" FROM unifi_clients WHERE time >= $1"#, self.since
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
            r#"SELECT name, id, time, state as "state: ApplianceStateType" FROM appliances WHERE time >= $1"#, self.since
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| ApplianceEvent {
            time: r.time,
            id: r.id,
            state: r.state,
            name: r.name,
        })
        .collect_vec())
    }
}
