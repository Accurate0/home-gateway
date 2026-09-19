use crate::repo::WorkflowRepo;
use crate::repo::workflow::WorkflowRunStepRow;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

pub struct WorkflowRunStepsDataLoader {
    pub repo: WorkflowRepo,
}

impl Loader<i64> for WorkflowRunStepsDataLoader {
    type Value = Vec<WorkflowRunStepRow>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let rows = self.repo.run_steps(keys).await?;

        let mut steps: HashMap<i64, Vec<WorkflowRunStepRow>> = HashMap::new();

        for row in rows {
            steps.entry(row.run_id).or_default().push(row);
        }

        Ok(steps)
    }
}
