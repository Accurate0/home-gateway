use crate::repo::RepoRegistry;
use async_graphql::{InputObject, Object, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use uuid::Uuid;

#[derive(InputObject)]
pub struct EnergyHistoryInput {
    pub since: DateTime<Utc>,
}

#[derive(InputObject)]
pub struct EnergyGapsInput {
    pub since: DateTime<Utc>,
    pub interval_seconds: i64,
}

pub struct EnergyObject {}

#[derive(SimpleObject, Debug)]
pub struct EnergyGap {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub missing_intervals: i64,
}

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

    pub async fn gaps(
        &self,
        ctx: &async_graphql::Context<'_>,
        input: EnergyGapsInput,
    ) -> async_graphql::Result<Vec<EnergyGap>> {
        if input.interval_seconds <= 0 {
            return Err("intervalSeconds must be positive".into());
        }

        let repos = ctx.data::<RepoRegistry>()?;
        let interval = chrono::TimeDelta::seconds(input.interval_seconds);

        Ok(repos
            .energy()
            .gaps_since(input.since, interval)
            .await?
            .into_iter()
            .map(|gap| EnergyGap {
                start: gap.start,
                end: gap.end,
                missing_intervals: (gap.end - gap.start).num_seconds() / input.interval_seconds - 1,
            })
            .collect_vec())
    }
}
