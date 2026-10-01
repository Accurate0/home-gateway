use std::sync::Arc;

use crate::auth::AuthLockout;
use crate::auth::api_types::{ApiKeyInfo, CreatedKey};
use crate::repo::ApiKeyRepo;
use crate::repo::api_key::{ApiKeyChanges, ApiKeyRow, NewApiKey};
use crate::settings::AuthSettings;
use chrono::{DateTime, Utc};
use moka::future::Cache;
use uuid::Uuid;

use axum::http::StatusCode;

use super::{AuthContext, GeneratedKey, OAuthValidator};

#[derive(Clone)]
pub struct AuthManager {
    keys: ApiKeyRepo,
    cache: Cache<String, Option<Arc<ApiKeyRow>>>,
    touched: Cache<Uuid, ()>,
    lockout: AuthLockout,
    oauth: Option<Arc<OAuthValidator>>,
}

impl AuthManager {
    pub fn new(
        keys: ApiKeyRepo,
        oauth: Option<Arc<OAuthValidator>>,
        settings: &AuthSettings,
    ) -> Self {
        let cache = Cache::builder()
            .max_capacity(settings.api_key_cache.capacity)
            .time_to_live(settings.api_key_cache.ttl())
            .build();

        let touched = Cache::builder()
            .max_capacity(settings.api_key_cache.capacity)
            .time_to_live(settings.last_used_interval())
            .build();

        Self {
            keys,
            cache,
            touched,
            lockout: AuthLockout::new(&settings.lockout),
            oauth,
        }
    }

    pub fn lockout(&self) -> &AuthLockout {
        &self.lockout
    }

    pub async fn validate_oauth(&self, token: &str) -> Option<Result<AuthContext, StatusCode>> {
        let oauth = self.oauth.as_ref()?;

        Some(oauth.validate(token).await)
    }

    pub async fn lookup_by_hash(
        &self,
        hash: &str,
    ) -> Result<Option<Arc<ApiKeyRow>>, Arc<sqlx::Error>> {
        let keys = self.keys.clone();
        let hash = hash.to_owned();

        self.cache
            .try_get_with(hash.clone(), async move {
                let row = keys.find_by_hash(&hash).await?;

                Ok(row.map(Arc::new))
            })
            .await
    }

    pub async fn touch_last_used(&self, id: Uuid) {
        if !self.touched.entry(id).or_insert(()).await.is_fresh() {
            return;
        }

        let keys = self.keys.clone();

        tokio::spawn(async move {
            if let Err(e) = keys.touch_last_used(id).await {
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
        let generated = GeneratedKey::generate();

        let id = self
            .keys
            .insert(NewApiKey {
                name,
                key_prefix: &generated.key_prefix,
                key_hash: &generated.key_hash,
                scopes,
                expires_at,
            })
            .await?;

        self.cache.invalidate(&generated.key_hash).await;

        Ok(CreatedKey {
            id,
            name: name.to_owned(),
            key_prefix: generated.key_prefix,
            scopes: scopes.to_vec(),
            expires_at,
            key: generated.key,
        })
    }

    pub async fn list(&self) -> Result<Vec<ApiKeyInfo>, sqlx::Error> {
        self.keys.list().await
    }

    pub async fn update(
        &self,
        id: Uuid,
        changes: ApiKeyChanges<'_>,
    ) -> Result<Option<ApiKeyInfo>, sqlx::Error> {
        let Some(updated) = self.keys.update(id, changes).await? else {
            return Ok(None);
        };

        self.cache.invalidate(&updated.key_hash).await;

        Ok(Some(updated.info))
    }

    pub async fn claim(
        &self,
        name: &str,
        scopes: &[String],
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<bool, sqlx::Error> {
        let hash = self.keys.claim(name, scopes, expires_at).await?;

        if let Some(hash) = &hash {
            self.cache.invalidate(hash).await;
        }

        Ok(hash.is_some())
    }

    pub async fn regenerate(&self, id: Uuid) -> Result<Option<CreatedKey>, sqlx::Error> {
        let generated = GeneratedKey::generate();

        let rotated = self
            .keys
            .rotate(id, &generated.key_prefix, &generated.key_hash)
            .await?;

        let Some(rotated) = rotated else {
            return Ok(None);
        };

        self.cache.invalidate(&rotated.old_hash).await;
        self.cache.invalidate(&generated.key_hash).await;

        Ok(Some(CreatedKey {
            id,
            name: rotated.name,
            key_prefix: generated.key_prefix,
            scopes: rotated.scopes,
            expires_at: rotated.expires_at,
            key: generated.key,
        }))
    }

    pub async fn revoke(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let hash = self.keys.revoke(id).await?;

        if let Some(hash) = &hash {
            self.cache.invalidate(hash).await;
        }

        Ok(hash.is_some())
    }
}
