use async_graphql::{InputObject, Object, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(InputObject)]
pub struct EnergyHistoryInput {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

pub struct EnergyObject {}

#[derive(serde::Serialize, serde::Deserialize, SimpleObject, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnergyConsumption {
    pub id: Uuid,
    pub used: f64,
    pub solar_exported: f64,
    pub time: DateTime<Utc>,
}

#[Object]
impl EnergyObject {
    pub async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        input: EnergyHistoryInput,
    ) -> async_graphql::Result<Vec<EnergyConsumption>> {
        let from = input.from;
        let to = input.to;
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(sqlx::query!(
            "SELECT * FROM energy_consumption WHERE time < $1 AND time >= $2 ORDER BY time ASC",
            to,
            from
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| EnergyConsumption {
            id: r.id,
            time: r.time,
            used: r.energy_used,
            solar_exported: r.solar_exported,
        })
        .collect_vec())
    }
}
