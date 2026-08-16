use home_gateway::auth::{AuthManager, hash_key};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::common::db::fresh_database;

async fn manager() -> AuthManager {
    AuthManager::new(fresh_database().await.pool, None)
}

#[tokio::test]
async fn claim_updates_scopes_by_name() {
    let mgr = manager().await;
    let created = mgr
        .create("svc", &["graphql:solar:read".to_owned()], None)
        .await
        .unwrap();

    assert!(
        mgr.claim("svc", &["rest:epd:read".to_owned()], None)
            .await
            .unwrap()
    );

    let looked_up = mgr
        .lookup_by_hash(&hash_key(&created.key))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(looked_up.scopes, ["rest:epd:read"]);

    assert!(!mgr.claim("missing", &[], None).await.unwrap());
}

#[tokio::test]
async fn regenerate_swaps_the_token() {
    let mgr = manager().await;
    let created = mgr
        .create("svc", &["graphql:solar:read".to_owned()], None)
        .await
        .unwrap();
    let old_hash = hash_key(&created.key);

    let regenerated = mgr.regenerate(created.id).await.unwrap().unwrap();
    assert_eq!(regenerated.id, created.id);
    assert_eq!(regenerated.name, "svc");
    assert_eq!(regenerated.scopes, ["graphql:solar:read"]);
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
async fn regenerate_missing_returns_none() {
    let mgr = manager().await;

    assert!(mgr.regenerate(Uuid::new_v4()).await.unwrap().is_none());
}
