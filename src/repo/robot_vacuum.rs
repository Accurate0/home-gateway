use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use tracing::Instrument;
use uuid::Uuid;

#[derive(Clone)]
pub struct RobotVacuumRepo {
    db: Pool<Postgres>,
}

#[derive(Clone)]
pub struct RobotVacuumStateRow {
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

impl RobotVacuumRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn record_valetudo_state(
        &self,
        event_id: Uuid,
        device_id: &str,
        state: Option<&str>,
        battery_level: Option<i32>,
        fan_speed: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.db.begin().await?;

        sqlx::query!(
            "INSERT INTO robot_vacuum_events (event_id, device_id, source, state, battery_level) \
             VALUES ($1, $2, 'valetudo', $3, $4)",
            event_id,
            device_id,
            state,
            battery_level,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO latest_robot_vacuum_state (device_id, source, state, battery_level, fan_speed) \
             VALUES ($1, 'valetudo', $2, $3, $4) \
             ON CONFLICT (device_id) DO UPDATE SET \
               source = EXCLUDED.source, \
               state = COALESCE(EXCLUDED.state, latest_robot_vacuum_state.state), \
               battery_level = COALESCE(EXCLUDED.battery_level, latest_robot_vacuum_state.battery_level), \
               fan_speed = COALESCE(EXCLUDED.fan_speed, latest_robot_vacuum_state.fan_speed), \
               updated_at = now()",
            device_id,
            state,
            battery_level,
            fan_speed,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    pub async fn upsert_valetudo_attributes(
        &self,
        device_id: &str,
        current_clean_area: Option<f64>,
        clean_count: Option<i32>,
        attributes: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO latest_robot_vacuum_state (device_id, source, current_clean_area, clean_count, attributes) \
             VALUES ($1, 'valetudo', $2, $3, $4) \
             ON CONFLICT (device_id) DO UPDATE SET \
               current_clean_area = COALESCE(EXCLUDED.current_clean_area, latest_robot_vacuum_state.current_clean_area), \
               clean_count = COALESCE(EXCLUDED.clean_count, latest_robot_vacuum_state.clean_count), \
               attributes = EXCLUDED.attributes, \
               updated_at = now()",
            device_id,
            current_clean_area,
            clean_count,
            attributes,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn record_roborock_status(
        &self,
        event_id: Uuid,
        device_id: &str,
        state: &str,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.db.begin().await?;

        sqlx::query!(
            "INSERT INTO robot_vacuum_events (event_id, device_id, source, state) \
             VALUES ($1, $2, 'roborock', $3)",
            event_id,
            device_id,
            state,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO latest_robot_vacuum_state (device_id, source, state) \
             VALUES ($1, 'roborock', $2) \
             ON CONFLICT (device_id) DO UPDATE SET \
               source = EXCLUDED.source, state = EXCLUDED.state, updated_at = now()",
            device_id,
            state,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    pub async fn record_roborock_battery(
        &self,
        event_id: Uuid,
        device_id: &str,
        level: Option<i32>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.db.begin().await?;

        sqlx::query!(
            "INSERT INTO robot_vacuum_events (event_id, device_id, source, battery_level) \
             VALUES ($1, $2, 'roborock', $3)",
            event_id,
            device_id,
            level,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO latest_robot_vacuum_state (device_id, source, battery_level) \
             VALUES ($1, 'roborock', $2) \
             ON CONFLICT (device_id) DO UPDATE SET \
               source = EXCLUDED.source, battery_level = EXCLUDED.battery_level, updated_at = now()",
            device_id,
            level,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    pub async fn upsert_roborock_room(
        &self,
        device_id: &str,
        room: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO latest_robot_vacuum_state (device_id, source, room) \
             VALUES ($1, 'roborock', $2) \
             ON CONFLICT (device_id) DO UPDATE SET \
               source = EXCLUDED.source, room = EXCLUDED.room, updated_at = now()",
            device_id,
            room,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
    pub async fn latest_many(
        &self,
        keys: &[String],
    ) -> Result<Vec<RobotVacuumStateRow>, sqlx::Error> {
        sqlx::query_as!(
            RobotVacuumStateRow,
            r#"
            SELECT device_id, source, state, battery_level, fan_speed, current_clean_area, clean_count, room, updated_at
            FROM latest_robot_vacuum_state
            WHERE device_id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&self.db)
        .instrument(tracing::info_span!("bulk-get-robot-vacuum-state"))
        .await
    }
}
