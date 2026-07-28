use async_graphql::dataloader::Loader;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use tracing::Instrument;

pub struct RobotVacuumStateDataLoader {
    pub database: Pool<Postgres>,
}

#[derive(Clone)]
pub struct RobotVacuumStateModel {
    pub device_id: String,
    pub source: String,
    pub state: Option<String>,
    pub battery_level: Option<i32>,
    pub fan_speed: Option<String>,
    pub current_clean_area: Option<f64>,
    pub clean_count: Option<i32>,
    pub room: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl Loader<String> for RobotVacuumStateDataLoader {
    type Value = RobotVacuumStateModel;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let rows = sqlx::query_as!(
            RobotVacuumStateModel,
            r#"
            SELECT device_id, source, state, battery_level, fan_speed, current_clean_area, clean_count, room, updated_at
            FROM latest_robot_vacuum_state
            WHERE device_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.database)
        .instrument(tracing::info_span!("bulk-get-robot-vacuum-state"))
        .await?;

        Ok(rows.into_iter().map(|r| (r.device_id.clone(), r)).collect())
    }
}
