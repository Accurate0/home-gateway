pub mod types;

use chrono::TimeDelta;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub use types::{
    DeviceIntent, DeviceKind, DeviceReport, IntentAttributes, IntentStatus, PowerState,
    SwitchAttributes,
};

#[derive(Clone)]
pub struct IntentRepo {
    db: Pool<Postgres>,
}

fn decode(
    id: i64,
    kind: &str,
    attributes: serde_json::Value,
) -> Option<(DeviceKind, IntentAttributes)> {
    let Some(kind) = DeviceKind::parse(kind) else {
        tracing::warn!("intent {id} has unknown device kind `{kind}`, skipping it");

        return None;
    };

    match serde_json::from_value::<IntentAttributes>(attributes) {
        Ok(attributes) => Some((kind, attributes)),
        Err(e) => {
            tracing::warn!("intent {id} has undecodable attributes, skipping it: {e}");

            None
        }
    }
}

impl IntentRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn replace_pending(
        &self,
        address: &str,
        attributes: &IntentAttributes,
        relative: bool,
        event_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let kind = attributes.kind();
        let payload = serde_json::to_value(attributes).unwrap_or(serde_json::Value::Null);

        let mut tx = self.db.begin().await?;

        sqlx::query!(
            "UPDATE device_intent SET status = 'superseded', settled_at = now() \
             WHERE kind = $1 AND address = $2 AND status = 'pending'",
            kind.as_str(),
            address,
        )
        .execute(&mut *tx)
        .await?;

        let id = sqlx::query_scalar!(
            "INSERT INTO device_intent (kind, address, attributes, relative, status, event_id) \
             VALUES ($1, $2, $3, $4, 'pending', $5) RETURNING id",
            kind.as_str(),
            address,
            payload,
            relative,
            event_id,
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(id)
    }

    pub async fn claim_due(
        &self,
        grace: TimeDelta,
        backoff: TimeDelta,
        limit: i64,
    ) -> Result<Vec<DeviceIntent>, sqlx::Error> {
        let rows = sqlx::query!(
            "UPDATE device_intent SET attempts = attempts + 1, last_attempt_at = now() \
             WHERE id IN ( \
                 SELECT id FROM device_intent \
                  WHERE status = 'pending' \
                    AND relative = false \
                    AND requested_at < now() - make_interval(secs => $1) \
                    AND (last_attempt_at IS NULL \
                         OR last_attempt_at < now() - make_interval(secs => $2)) \
                  ORDER BY last_attempt_at NULLS FIRST \
                  LIMIT $3 \
                  FOR UPDATE SKIP LOCKED \
             ) \
             RETURNING id, kind, address, attributes, relative, attempts, event_id, requested_at",
            grace.num_seconds() as f64,
            backoff.num_seconds() as f64,
            limit,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let (_, attributes) = decode(row.id, &row.kind, row.attributes)?;

                Some(DeviceIntent {
                    id: row.id,
                    address: row.address,
                    attributes,
                    relative: row.relative,
                    attempts: row.attempts,
                    event_id: row.event_id,
                    requested_at: row.requested_at,
                })
            })
            .collect())
    }

    pub async fn pending_for(
        &self,
        kind: DeviceKind,
        address: &str,
    ) -> Result<Vec<DeviceIntent>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, kind, address, attributes, relative, attempts, event_id, requested_at \
             FROM device_intent \
             WHERE kind = $1 AND address = $2 AND status = 'pending' \
             ORDER BY requested_at",
            kind.as_str(),
            address,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let (_, attributes) = decode(row.id, &row.kind, row.attributes)?;

                Some(DeviceIntent {
                    id: row.id,
                    address: row.address,
                    attributes,
                    relative: row.relative,
                    attempts: row.attempts,
                    event_id: row.event_id,
                    requested_at: row.requested_at,
                })
            })
            .collect())
    }

    pub async fn settle(&self, ids: &[i64], status: IntentStatus) -> Result<(), sqlx::Error> {
        if ids.is_empty() {
            return Ok(());
        }

        sqlx::query!(
            "UPDATE device_intent SET status = $2, settled_at = now() \
             WHERE id = ANY($1) AND status = 'pending'",
            ids,
            status.as_str(),
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
