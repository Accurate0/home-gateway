use chrono::TimeDelta;
use home_gateway::auth::{AuthManager, ExpiryChange, hash_key};
use home_gateway::repo::ApiKeyRepo;
use home_gateway::repo::api_key::ApiKeyChanges;
use home_gateway::settings::{AuthLockoutSettings, AuthSettings, CacheSettings};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::common::db::fresh_database;

async fn manager() -> AuthManager {
    AuthManager::new(
        ApiKeyRepo::new(fresh_database().await.pool),
        None,
        &AuthSettings {
            api_key_cache: CacheSettings {
                capacity: 1024,
                ttl: TimeDelta::hours(1),
            },
            last_used_interval: TimeDelta::minutes(1),
            lockout: AuthLockoutSettings {
                attempts: 20,
                window: TimeDelta::minutes(5),
                capacity: 1024,
            },
            api_keys: Vec::new(),
            oauth: None,
        },
    )
}

#[tokio::test]
async fn claim_updates_scopes_by_name() {
    let mgr = manager().await;
    let created = mgr
        .create("svc", &["solar:read".to_owned()], None)
        .await
        .unwrap();

    assert!(
        mgr.claim("svc", &["epd:read".to_owned()], None)
            .await
            .unwrap()
    );

    let looked_up = mgr
        .lookup_by_hash(&hash_key(&created.key))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(looked_up.scopes, ["epd:read"]);

    assert!(!mgr.claim("missing", &[], None).await.unwrap());
}

#[tokio::test]
async fn regenerate_swaps_the_token() {
    let mgr = manager().await;
    let created = mgr
        .create("svc", &["solar:read".to_owned()], None)
        .await
        .unwrap();
    let old_hash = hash_key(&created.key);

    let regenerated = mgr.regenerate(created.id).await.unwrap().unwrap();
    assert_eq!(regenerated.id, created.id);
    assert_eq!(regenerated.name, "svc");
    assert_eq!(regenerated.scopes, ["solar:read"]);
    assert_ne!(regenerated.key, created.key);

    assert!(
        mgr.lookup_by_hash(&old_hash).await.unwrap().is_none(),
        "old token must stop authenticating"
    );
    assert!(
        mgr.lookup_by_hash(&hash_key(&regenerated.key))
            .await
            .unwrap()
            .is_some(),
        "new token must authenticate"
    );
}

#[tokio::test]
async fn regenerate_of_a_revoked_key_returns_none() {
    let mgr = manager().await;
    let created = mgr.create("svc", &[], None).await.unwrap();

    assert!(mgr.revoke(created.id).await.unwrap());

    assert!(mgr.regenerate(created.id).await.unwrap().is_none());
}

#[tokio::test]
async fn update_sets_keeps_and_clears_the_expiry() {
    let mgr = manager().await;
    let expiry = chrono::Utc::now() + TimeDelta::days(1);
    let created = mgr.create("svc", &[], Some(expiry)).await.unwrap();

    let renamed = mgr
        .update(
            created.id,
            ApiKeyChanges {
                name: Some("renamed"),
                scopes: None,
                expires_at: ExpiryChange::Keep,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(renamed.name, "renamed");
    assert!(renamed.expires_at.is_some(), "an absent expiry is kept");

    let cleared = mgr
        .update(
            created.id,
            ApiKeyChanges {
                name: None,
                scopes: None,
                expires_at: ExpiryChange::Clear,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(cleared.name, "renamed");
    assert_eq!(cleared.expires_at, None);
}

#[tokio::test]
async fn regenerate_missing_returns_none() {
    let mgr = manager().await;

    assert!(mgr.regenerate(Uuid::new_v4()).await.unwrap().is_none());
}
