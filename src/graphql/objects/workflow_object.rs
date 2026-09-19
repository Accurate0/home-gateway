use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Json, SimpleObject};
use chrono::{DateTime, Utc};

use crate::graphql::dataloader::workflow_run_steps::WorkflowRunStepsDataLoader;
use crate::mode::Mode;
use crate::repo::workflow::WorkflowRunStepRow;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct WorkflowRun {
    pub id: async_graphql::ID,
    #[graphql(skip)]
    pub run_id: i64,
    pub slug: String,
    pub name: String,
    pub event_id: String,
    pub outcome: String,
    pub dry_run: bool,
    pub duration_ms: i64,
    pub error: Option<String>,
    pub trigger: Option<Json<serde_json::Value>>,
    pub started_at: DateTime<Utc>,
}

#[ComplexObject]
impl WorkflowRun {
    async fn steps(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<WorkflowRunStep>> {
        let loader = ctx.data::<DataLoader<WorkflowRunStepsDataLoader>>()?;
        let rows = loader.load_one(self.run_id).await?.unwrap_or_default();

        Ok(rows.into_iter().map(WorkflowRunStep::from).collect())
    }
}

#[derive(SimpleObject)]
pub struct WorkflowRunStep {
    pub seq: i32,
    pub depth: i32,
    pub kind: String,
    pub outcome: String,
    pub guard: Option<String>,
    pub detail: Option<String>,
    pub error: Option<String>,
    pub duration_ms: i64,
    pub at: DateTime<Utc>,
}

impl From<WorkflowRunStepRow> for WorkflowRunStep {
    fn from(row: WorkflowRunStepRow) -> Self {
        WorkflowRunStep {
            seq: row.seq,
            depth: i32::from(row.depth),
            kind: row.kind,
            outcome: row.outcome,
            guard: row.guard,
            detail: row.detail,
            error: row.error,
            duration_ms: row.duration_ms,
            at: row.at,
        }
    }
}

#[derive(SimpleObject)]
pub struct WorkflowStatus {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub group: String,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub config_enabled: bool,
    pub dry_run: bool,
    pub reusable: bool,
    pub modes: Vec<Mode>,
}
