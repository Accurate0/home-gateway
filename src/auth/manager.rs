use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use home_gateway::api_types::{ApiKeyInfo, CreatedKey};
use moka::future::Cache;
use rand::{Rng, distr::Alphanumeric};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use super::hash_key;

const KEY_PREFIX: &str = "hg_";
const KEY_RANDOM_LEN: usize = 40;
const CACHE_CAPACITY: u64 = 1024;
const CACHE_TTL: Duration = Duration::from_secs(3600);

#[derive(Debug, Clone)]
pub struct CachedKey {
    pub id: Uuid,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct AuthManager {
    db: Pool<Postgres>,
    cache: Cache<String, Option<Arc<CachedKey>>>,
}

impl AuthManager {
    pub fn new(db: Pool<Postgres>) -> Self {
        let cache = Cache::builder()
            .max_capacity(CACHE_CAPACITY)
            .time_to_live(CACHE_TTL)
            .build();

        Self { db, cache }
    }

    pub async fn lookup_by_hash(
        &self,
        hash: &str,
    ) -> Result<Option<Arc<CachedKey>>, Arc<sqlx::Error>> {
        let db = self.db.clone();
        let hash = hash.to_owned();

        self.cache
            .try_get_with(hash.clone(), async move {
                let row = sqlx::query_as!(
                    CachedKey,
                    "SELECT id, scopes, expires_at, revoked_at FROM api_keys WHERE key_hash = $1",
                    hash
                )
                .fetch_optional(&db)
                .await?;

                Ok(row.map(Arc::new))
            })
            .await
    }

    pub fn touch_last_used(&self, id: Uuid) {
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Err(e) =
                sqlx::query!("UPDATE api_keys SET last_used_at = now() WHERE id = $1", id)
                    .execute(&db)
                    .await
            {
                tracing::warn!("failed to update api key last_used_at: {e}");
            }
        });
    }

    pub async fn create(
        &self,
        name: &str,
        scopes: &[String],
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<CreatedKey, sqlx::Error> {
        let random: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(KEY_RANDOM_LEN)
            .map(char::from)
            .collect();
        let key = format!("{KEY_PREFIX}{random}");
        let key_prefix = format!("{KEY_PREFIX}{}", &random[..6]);
        let key_hash = hash_key(&key);

        let id = sqlx::query_scalar!(
            "INSERT INTO api_keys (name, key_prefix, key_hash, scopes, expires_at) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
            name,
            key_prefix,
            key_hash,
            scopes,
            expires_at
        )
        .fetch_one(&self.db)
        .await?;

        self.cache.invalidate(&key_hash).await;

        Ok(CreatedKey {
            id,
            name: name.to_owned(),
            key_prefix,
            scopes: scopes.to_vec(),
            expires_at,
            key,
        })
    }

    pub async fn list(&self) -> Result<Vec<ApiKeyInfo>, sqlx::Error> {
        sqlx::query_as!(
            ApiKeyInfo,
            "SELECT id, name, key_prefix, scopes, created_at, last_used_at, expires_at, revoked_at \
             FROM api_keys ORDER BY created_at DESC"
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        scopes: Option<&[String]>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Option<ApiKeyInfo>, sqlx::Error> {
        let row = sqlx::query!(
            "UPDATE api_keys SET \
               name = COALESCE($2, name), \
               scopes = COALESCE($3, scopes), \
               expires_at = COALESCE($4, expires_at) \
             WHERE id = $1 AND revoked_at IS NULL \
             RETURNING id, name, key_prefix, key_hash, scopes, created_at, last_used_at, expires_at, revoked_at",
            id,
            name,
            scopes,
            expires_at
        )
        .fetch_optional(&self.db)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        self.cache.invalidate(&row.key_hash).await;

        Ok(Some(ApiKeyInfo {
            id: row.id,
            name: row.name,
            key_prefix: row.key_prefix,
            scopes: row.scopes,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
            expires_at: row.expires_at,
            revoked_at: row.revoked_at,
        }))
    }

    pub async fn revoke(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let hash: Option<String> = sqlx::query_scalar!(
            "UPDATE api_keys SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL \
             RETURNING key_hash",
            id
        )
        .fetch_optional(&self.db)
        .await?;

        if let Some(hash) = &hash {
            self.cache.invalidate(hash).await;
        }

        Ok(hash.is_some())
    }
}
