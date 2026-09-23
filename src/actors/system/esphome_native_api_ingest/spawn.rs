use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, FactoryMessage, queues, routing},
};

use super::{EsphomeNativeApiIngest, EsphomeNativeApiIngestBuilder, Message};
use crate::state::AppState;

pub async fn spawn_esphome_native_api_ingest(
    root_supervisor_ref: &ActorRef<crate::actors::root::RootMessage>,
    shared_actor_state: AppState,
) -> anyhow::Result<ActorRef<FactoryMessage<String, Message>>> {
    let factory_def = Factory::<
        String,
        Message,
        (),
        EsphomeNativeApiIngest,
        routing::KeyPersistentRouting<String, Message>,
        queues::DefaultQueue<String, Message>,
    >::default();

    let workers = shared_actor_state
        .settings
        .actors
        .workers
        .esphome_native_api_ingest;

    let factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(EsphomeNativeApiIngestBuilder {
            shared_actor_state,
        }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(workers)
        .build();

    let (actor_ref, _) = root_supervisor_ref
        .spawn_linked(
            Some(EsphomeNativeApiIngest::NAME.to_string()),
            factory_def,
            factory_args,
        )
        .await?;

    Ok(actor_ref)
}
