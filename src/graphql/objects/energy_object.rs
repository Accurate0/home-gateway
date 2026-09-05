use crate::repo::RepoRegistry;
use async_graphql::{InputObject, Object, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use uuid::Uuid;

#[derive(InputObject)]
pub struct EnergyHistoryInput {
    pub since: DateTime<Utc>,
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
        let repos = ctx.data::<RepoRegistry>()?;

        Ok(repos
            .energy()
            .history_since(input.since)
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
