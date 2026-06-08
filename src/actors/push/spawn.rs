use std::sync::Arc;

use crate::{http::get_traced_http_client, types::SharedActorState};
use gcp_auth::{CustomServiceAccount, TokenProvider};
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, FactoryMessage, queues, routing},
};

use super::{PushMessage, PushWorker, PushWorkerBuilder};

pub async fn spawn_push(
    root_supervisor_ref: &ActorRef<()>,
    shared_actor_state: SharedActorState,
) -> anyhow::Result<ActorRef<FactoryMessage<(), PushMessage>>> {
    let push_factory_def = Factory::<
        (),
        PushMessage,
        (),
        PushWorker,
        routing::QueuerRouting<(), PushMessage>,
        queues::DefaultQueue<(), PushMessage>,
    >::default();

    let client = get_traced_http_client()?;

    // Load the FCM service account once at startup. The secret holds the raw
    // service-account JSON; a missing/invalid value is logged rather than fatal
    // so the rest of the gateway still comes up.
    let sa_json = shared_actor_state
        .settings
        .load()
        .fcm_service_account_json
        .clone();
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

    let push_factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(PushWorkerBuilder {
            client,
            shared_actor_state,
            token_provider,
        }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(1)
        .build();

    let (actor_ref, _) = root_supervisor_ref
        .spawn_linked(
            Some(PushWorker::NAME.to_string()),
            push_factory_def,
            push_factory_args,
        )
        .await?;

    Ok(actor_ref)
}
