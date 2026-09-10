use chrono::{DateTime, TimeDelta, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::repo::notification_interaction::NotificationInteraction;

#[derive(Clone)]
pub struct PushRepo {
    db: Pool<Postgres>,
}

pub struct NewPushNotification<'a> {
    pub tag: &'a str,
    pub title: &'a str,
    pub body: &'a str,
    pub category: &'a str,
    pub actions: serde_json::Value,
    pub remind_after: TimeDelta,
    pub reminders: u32,
}

#[derive(Debug, Clone)]
pub struct PushNotificationRow {
    pub id: Uuid,
    pub tag: String,
    pub title: String,
    pub body: String,
    pub category: String,
    pub actions: serde_json::Value,
    pub send_count: i32,
    pub next_reminder_at: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PushNotificationInteractionRow {
    pub notification_id: Uuid,
    pub kind: String,
    pub at: DateTime<Utc>,
}

impl PushRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.push.tokens", err)]
    pub async fn tokens(&self) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar!("SELECT token FROM android_push_tokens")
            .fetch_all(&self.db)
            .await
    }

    #[tracing::instrument(skip_all, name = "db.push.upsert_token", err)]
    pub async fn upsert_token(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO android_push_tokens (token) VALUES ($1) ON CONFLICT (token) DO UPDATE SET updated_at = now()",
            token
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.push.delete_token", err)]
    pub async fn delete_token(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM android_push_tokens WHERE token = $1", token)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.push.record_sent", err)]
    pub async fn record_sent(
        &self,
        notification: NewPushNotification<'_>,
    ) -> Result<PushNotificationRow, sqlx::Error> {
        let NewPushNotification {
            tag,
            title,
            body,
            category,
            actions,
            remind_after,
            reminders,
        } = notification;

        let max_reminders = i32::try_from(reminders).unwrap_or(i32::MAX);

        sqlx::query_as!(
            PushNotificationRow,
            "INSERT INTO push_notification \
             (tag, title, body, category, actions, remind_after_secs, max_reminders, next_reminder_at) \
             VALUES ($1, $2, $3, $4, $5, $6::BIGINT, $7::INT, \
             CASE WHEN $7::INT > 0 THEN now() + make_interval(secs => $6::BIGINT::DOUBLE PRECISION) END) \
             RETURNING id, tag, title, body, category, actions, send_count, next_reminder_at, \
             acknowledged_at, created_at",
            tag,
            title,
            body,
            category,
            actions,
            remind_after.num_seconds(),
            max_reminders,
        )
        .fetch_one(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.push.take_due_reminder", err)]
    pub async fn take_due_reminder(
        &self,
        id: Uuid,
    ) -> Result<Option<PushNotificationRow>, sqlx::Error> {
        sqlx::query_as!(
            PushNotificationRow,
            "UPDATE push_notification SET \
             send_count = send_count + 1, \
             next_reminder_at = CASE WHEN send_count < max_reminders \
             THEN now() + make_interval(secs => remind_after_secs::DOUBLE PRECISION) END \
             WHERE id = $1 AND acknowledged_at IS NULL AND next_reminder_at IS NOT NULL \
             AND next_reminder_at <= now() + INTERVAL '5 seconds' \
             RETURNING id, tag, title, body, category, actions, send_count, next_reminder_at, \
             acknowledged_at, created_at",
            id,
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.push.clear_reminder", err)]
    pub async fn clear_reminder(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE push_notification SET next_reminder_at = NULL WHERE id = $1",
            id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.push.pending_reminders", err)]
    pub async fn pending_reminders(&self) -> Result<Vec<PushNotificationRow>, sqlx::Error> {
        sqlx::query_as!(
            PushNotificationRow,
            "SELECT id, tag, title, body, category, actions, send_count, next_reminder_at, \
             acknowledged_at, created_at \
             FROM push_notification \
             WHERE acknowledged_at IS NULL AND next_reminder_at IS NOT NULL \
             ORDER BY next_reminder_at"
        )
        .fetch_all(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.push.record_interaction", err)]
    pub async fn record_interaction(
        &self,
        id: Uuid,
        kind: NotificationInteraction,
    ) -> Result<bool, sqlx::Error> {
        let mut tx = self.db.begin().await?;

        let exists = sqlx::query_scalar!(
            "SELECT id FROM push_notification WHERE id = $1 FOR UPDATE",
            id
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some();

        if !exists {
            return Ok(false);
        }

        sqlx::query!(
            "INSERT INTO push_notification_interaction (notification_id, kind) VALUES ($1, $2)",
            id,
            kind.as_str(),
        )
        .execute(&mut *tx)
        .await?;

        if kind == NotificationInteraction::Acknowledged {
            sqlx::query!(
                "UPDATE push_notification SET \
                 acknowledged_at = COALESCE(acknowledged_at, now()), next_reminder_at = NULL \
                 WHERE id = $1",
                id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(true)
    }

    #[tracing::instrument(skip_all, name = "db.push.recent_notifications", err)]
    pub async fn recent_notifications(
        &self,
        limit: i64,
    ) -> Result<Vec<PushNotificationRow>, sqlx::Error> {
        sqlx::query_as!(
            PushNotificationRow,
            "SELECT id, tag, title, body, category, actions, send_count, next_reminder_at, \
             acknowledged_at, created_at \
             FROM push_notification ORDER BY created_at DESC LIMIT $1",
            limit
        )
        .fetch_all(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.push.interactions", err)]
    pub async fn interactions(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<PushNotificationInteractionRow>, sqlx::Error> {
        sqlx::query_as!(
            PushNotificationInteractionRow,
            "SELECT notification_id, kind, at FROM push_notification_interaction \
             WHERE notification_id = ANY($1) ORDER BY at",
            ids
        )
        .fetch_all(&self.db)
        .await
    }
}
