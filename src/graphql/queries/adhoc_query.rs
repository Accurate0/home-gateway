use std::collections::HashMap;

use async_graphql::Object;
use chrono::Utc;
use sqlx::{Pool, Postgres};

use crate::adhoc::task::checksum;
use crate::adhoc::{cron_registry, registry};
use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::adhoc_object::{AdhocCronTaskStatus, AdhocTaskStatus};

#[derive(Default)]
pub struct AdhocQuery;

#[Object]
impl AdhocQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::AdhocTask, Action::Read)))]
    async fn adhoc_cron_tasks(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<AdhocCronTaskStatus>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        let runs = sqlx::query!(
            "SELECT name, last_run_at, duration_ms, rows_affected, outcome FROM adhoc_cron_run"
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| {
            (
                r.name,
                (r.last_run_at, r.duration_ms, r.rows_affected, r.outcome),
            )
        })
        .collect::<HashMap<_, _>>();

        let now = Utc::now();

        Ok(cron_registry()
            .into_iter()
            .map(|task| {
                let schedule = task.schedule();
                let run = runs.get(task.name());

                AdhocCronTaskStatus {
                    id: task.name().to_owned(),
                    name: task.name().to_owned(),
                    schedule: schedule.expression(),
                    flag: task.flag().map(str::to_owned),
                    next_run_at: schedule
                        .time_until_next()
                        .ok()
                        .and_then(|delay| chrono::Duration::from_std(delay).ok())
                        .map(|delay| now + delay),
                    last_run_at: run.map(|r| r.0),
                    duration_ms: run.map(|r| r.1),
                    rows_affected: run.map(|r| r.2),
                    outcome: run.map(|r| r.3.clone()),
                }
            })
            .collect())
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::AdhocTask, Action::Read)))]
    async fn adhoc_tasks(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<AdhocTaskStatus>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        let runs =
            sqlx::query!("SELECT ordinal, checksum, completed_at, duration_ms FROM adhoc_task_run")
                .fetch_all(db)
                .await?
                .into_iter()
                .map(|r| (r.ordinal, (r.checksum, r.completed_at, r.duration_ms)))
                .collect::<HashMap<_, _>>();

        Ok(registry()
            .into_iter()
            .map(|task| {
                let run = runs.get(&task.ordinal());

                AdhocTaskStatus {
                    id: task.ordinal().to_string(),
                    ordinal: task.ordinal(),
                    name: task.name().to_owned(),
                    flag: task.flag().map(str::to_owned),
                    completed_at: run.map(|r| r.1),
                    duration_ms: run.map(|r| r.2),
                    pending: run.is_none(),
                    checksum_drifted: run.is_some_and(|r| r.0 != checksum(task.source())),
                }
            })
            .collect())
    }
}
