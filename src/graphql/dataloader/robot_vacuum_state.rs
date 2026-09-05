use crate::repo::RobotVacuumRepo;
use async_graphql::dataloader::Loader;
use std::{collections::HashMap, sync::Arc};

pub use crate::repo::robot_vacuum::RobotVacuumStateRow as RobotVacuumStateModel;

pub struct RobotVacuumStateDataLoader {
    pub repo: RobotVacuumRepo,
}

impl Loader<String> for RobotVacuumStateDataLoader {
    type Value = RobotVacuumStateModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = self.repo.latest_many(keys).await?;

        Ok(rows.into_iter().map(|r| (r.device_id.clone(), r)).collect())
    }
}
