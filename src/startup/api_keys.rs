use crate::auth::AuthManager;
use crate::state::AppState;

pub async fn reconcile(state: &AppState) {
    let auth = state.handles.expect::<AuthManager>();

    for key in &state.settings.auth.api_keys {
        match auth.claim(&key.name, &key.scopes, key.expires_at).await {
            Ok(true) => tracing::info!(name = %key.name, "reconciled api key scopes from config"),
            Ok(false) => tracing::warn!(
                name = %key.name,
                "config api key not found; mint it via the admin API"
            ),
            Err(e) => tracing::error!(name = %key.name, error = %e, "failed to reconcile api key"),
        }
    }
}
