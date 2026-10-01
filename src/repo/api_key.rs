use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::auth::ExpiryChange;
use crate::auth::api_types::ApiKeyInfo;

#[derive(Clone)]
pub struct ApiKeyRepo {
    db: Pool<Postgres>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyRow {
    pub id: Uuid,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub struct NewApiKey<'a> {
    pub name: &'a str,
    pub key_prefix: &'a str,
    pub key_hash: &'a str,
    pub scopes: &'a [String],
    pub expires_at: Option<DateTime<Utc>>,
}

pub struct ApiKeyChanges<'a> {
    pub name: Option<&'a str>,
    pub scopes: Option<&'a [String]>,
    pub expires_at: ExpiryChange,
}

pub struct UpdatedApiKey {
    pub key_hash: String,
    pub info: ApiKeyInfo,
}

pub struct RotatedApiKey {
    pub old_hash: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl ApiKeyRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.api_key.find_by_hash", err)]
    pub async fn find_by_hash(&self, hash: &str) -> Result<Option<ApiKeyRow>, sqlx::Error> {
        sqlx::query_as!(
            ApiKeyRow,
            "SELECT id, name, scopes, expires_at, revoked_at FROM api_keys WHERE key_hash = $1",
            hash
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.api_key.touch_last_used", err)]
    pub async fn touch_last_used(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE api_keys SET last_used_at = now() WHERE id = $1", id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.api_key.insert", err)]
    pub async fn insert(&self, key: NewApiKey<'_>) -> Result<Uuid, sqlx::Error> {
        sqlx::query_scalar!(
            "INSERT INTO api_keys (name, key_prefix, key_hash, scopes, expires_at) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
            key.name,
            key.key_prefix,
            key.key_hash,
            key.scopes,
            key.expires_at
        )
        .fetch_one(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.api_key.list", err)]
    pub async fn list(&self) -> Result<Vec<ApiKeyInfo>, sqlx::Error> {
        sqlx::query_as!(
            ApiKeyInfo,
            "SELECT id, name, key_prefix, scopes, created_at, last_used_at, expires_at, revoked_at \
             FROM api_keys ORDER BY created_at DESC"
        )
        .fetch_all(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.api_key.update", err)]
    pub async fn update(
        &self,
        id: Uuid,
        changes: ApiKeyChanges<'_>,
    ) -> Result<Option<UpdatedApiKey>, sqlx::Error> {
        let expires_at = match changes.expires_at {
            ExpiryChange::Keep | ExpiryChange::Clear => None,
            ExpiryChange::Set(at) => Some(at),
        };

        let row = sqlx::query!(
            "UPDATE api_keys SET \
               name = COALESCE($2, name), \
               scopes = COALESCE($3, scopes), \
               expires_at = CASE WHEN $4 THEN $5 ELSE expires_at END \
             WHERE id = $1 AND revoked_at IS NULL \
             RETURNING id, name, key_prefix, key_hash, scopes, created_at, last_used_at, expires_at, revoked_at",
            id,
            changes.name,
            changes.scopes,
            !changes.expires_at.is_keep(),
            expires_at
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| UpdatedApiKey {
            key_hash: row.key_hash,
            info: ApiKeyInfo {
                id: row.id,
                name: row.name,
                key_prefix: row.key_prefix,
                scopes: row.scopes,
                created_at: row.created_at,
                last_used_at: row.last_used_at,
                expires_at: row.expires_at,
                revoked_at: row.revoked_at,
            },
        }))
    }

    #[tracing::instrument(skip_all, name = "db.api_key.claim", err)]
    pub async fn claim(
        &self,
        name: &str,
        scopes: &[String],
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar!(
            "UPDATE api_keys SET \
               scopes = $2, \
               expires_at = COALESCE($3, expires_at) \
             WHERE name = $1 AND revoked_at IS NULL \
             RETURNING key_hash",
            name,
            scopes,
            expires_at
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.api_key.rotate", err)]
    pub async fn rotate(
        &self,
        id: Uuid,
        key_prefix: &str,
        key_hash: &str,
    ) -> Result<Option<RotatedApiKey>, sqlx::Error> {
        sqlx::query_as!(
            RotatedApiKey,
            r#"WITH previous AS (
                 SELECT id, key_hash FROM api_keys
                 WHERE id = $1 AND revoked_at IS NULL
                 FOR UPDATE
               )
               UPDATE api_keys SET key_prefix = $2, key_hash = $3
               FROM previous
               WHERE api_keys.id = previous.id
               RETURNING previous.key_hash AS "old_hash!", api_keys.name, api_keys.scopes, api_keys.expires_at"#,
            id,
            key_prefix,
            key_hash
        )
        .fetch_optional(&self.db)
        .await
    }

    #[tracing::instrument(skip_all, name = "db.api_key.revoke", err)]
    pub async fn revoke(&self, id: Uuid) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar!(
            "UPDATE api_keys SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL \
             RETURNING key_hash",
            id
        )
        .fetch_optional(&self.db)
        .await
    }
}
