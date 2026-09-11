use std::sync::Arc;

use crate::{http::get_traced_http_client, settings::HttpClientKind, state::AppState};
use gcp_auth::{CustomServiceAccount, TokenProvider};
use ractor::ActorRef;

use super::{PushActor, PushMessage};

pub async fn spawn_push(
    root_supervisor_ref: &ActorRef<crate::actors::root::RootMessage>,
    shared_actor_state: AppState,
) -> anyhow::Result<ActorRef<PushMessage>> {
    let client = get_traced_http_client(
        shared_actor_state
            .settings
            .http
            .clients
            .timeout_for(HttpClientKind::Push),
    )?;

    let sa_json = shared_actor_state.settings.fcm_service_account_json.clone();
    let token_provider: Option<Arc<dyn TokenProvider>> = if sa_json.is_empty() {
        tracing::warn!("fcm_service_account_json not set, android push disabled");
        None
    } else {
        match CustomServiceAccount::from_json(&sa_json) {
            Ok(sa) => Some(Arc::new(sa)),
            Err(e) => {
                tracing::error!("failed to load fcm service account: {e}");
                None
            }
        }
    };

    let (actor_ref, _) = root_supervisor_ref
        .spawn_linked(
            Some(PushActor::NAME.to_string()),
            PushActor {
                client,
                shared_actor_state,
                token_provider,
            },
            (),
        )
        .await?;

    Ok(actor_ref)
}
