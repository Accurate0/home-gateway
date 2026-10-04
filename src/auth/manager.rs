use std::sync::Arc;

use crate::auth::AuthLockout;
use crate::auth::api_types::{ApiKeyInfo, CreatedKey};
use crate::cache::MemoryCache;
use crate::repo::ApiKeyRepo;
use crate::repo::api_key::{ApiKeyChanges, ApiKeyRow, NewApiKey};
use crate::settings::AuthSettings;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use axum::http::StatusCode;

use super::{AuthContext, GeneratedKey, OAuthValidator};

#[derive(Clone)]
pub struct AuthManager {
    keys: ApiKeyRepo,
    cache: MemoryCache<String, Option<Arc<ApiKeyRow>>>,
    touched: MemoryCache<Uuid, ()>,
    lockout: AuthLockout,
    oauth: Option<Arc<OAuthValidator>>,
}

impl AuthManager {
    pub fn new(
        keys: ApiKeyRepo,
        oauth: Option<Arc<OAuthValidator>>,
        settings: &AuthSettings,
    ) -> Self {
        let cache = MemoryCache::builder("api_keys")
            .configure(|cache| {
                cache
                    .max_capacity(settings.api_key_cache.capacity)
                    .time_to_live(settings.api_key_cache.ttl())
            })
            .build();

        let touched = MemoryCache::builder("api_key_touched")
            .configure(|cache| {
                cache
                    .max_capacity(settings.api_key_cache.capacity)
                    .time_to_live(settings.last_used_interval())
            })
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

    #[tracing::instrument(
        name = "auth.validate_oauth",
        skip_all,
        fields(valid = tracing::field::Empty)
    )]
    pub async fn validate_oauth(&self, token: &str) -> Option<Result<AuthContext, StatusCode>> {
        let oauth = self.oauth.as_ref()?;

        let validated = oauth.validate(token).await;

        tracing::Span::current().record("valid", validated.is_ok());

        Some(validated)
    }

    #[tracing::instrument(
        name = "auth.lookup_by_hash",
        skip_all,
        fields(cached = tracing::field::Empty, found = tracing::field::Empty),
        err
    )]
    pub async fn lookup_by_hash(
        &self,
        hash: &str,
    ) -> Result<Option<Arc<ApiKeyRow>>, Arc<sqlx::Error>> {
        let keys = self.keys.clone();
        let key = hash.to_owned();
        let hash = hash.to_owned();

        let entry = self
            .cache
            .or_try_insert_with(key, async move {
                let row = keys.find_by_hash(&hash).await?;

                Ok::<_, sqlx::Error>(row.map(Arc::new))
            })
            .await?;

        let span = tracing::Span::current();

        span.record("cached", !entry.is_fresh());
        span.record("found", entry.value().is_some());

        Ok(entry.into_value())
    }

    #[tracing::instrument(
        name = "auth.touch_last_used",
        skip_all,
        fields(throttled = tracing::field::Empty)
    )]
    pub async fn touch_last_used(&self, id: Uuid) {
        let fresh = self.touched.or_insert(id, ()).await.is_fresh();

        tracing::Span::current().record("throttled", !fresh);

        if !fresh {
            return;
        }

        let keys = self.keys.clone();

        tokio::spawn(async move {
            if let Err(e) = keys.touch_last_used(id).await {
                tracing::warn!("failed to update api key last_used_at: {e}");
            }
        });
    }

    #[tracing::instrument(name = "auth.create", skip_all, err)]
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

    #[tracing::instrument(name = "auth.list", skip_all, err)]
    pub async fn list(&self) -> Result<Vec<ApiKeyInfo>, sqlx::Error> {
        self.keys.list().await
    }

    #[tracing::instrument(name = "auth.update", skip_all, err)]
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

    #[tracing::instrument(name = "auth.claim", skip_all, err)]
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

    #[tracing::instrument(name = "auth.regenerate", skip_all, err)]
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

    #[tracing::instrument(name = "auth.revoke", skip_all, err)]
    pub async fn revoke(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let hash = self.keys.revoke(id).await?;

        if let Some(hash) = &hash {
            self.cache.invalidate(hash).await;
        }

        Ok(hash.is_some())
    }
}
