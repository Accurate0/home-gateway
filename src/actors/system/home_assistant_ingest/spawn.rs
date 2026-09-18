use super::{HomeAssistantIngest, HomeAssistantIngestBuilder, Message};
use crate::state::AppState;
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, FactoryMessage, queues, routing},
};

pub async fn spawn_home_assistant_ingest(
    root_supervisor_ref: &ActorRef<crate::actors::root::RootMessage>,
    shared_actor_state: AppState,
) -> anyhow::Result<ActorRef<FactoryMessage<String, Message>>> {
    let factory_def = Factory::<
        String,
        Message,
        (),
        HomeAssistantIngest,
        routing::KeyPersistentRouting<String, Message>,
        queues::DefaultQueue<String, Message>,
    >::default();

    let workers = shared_actor_state
        .settings
        .actors
        .workers
        .home_assistant_ingest;

    let factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(HomeAssistantIngestBuilder { shared_actor_state }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(workers)
        .build();

    let (actor_ref, _) = root_supervisor_ref
        .spawn_linked(
            Some(HomeAssistantIngest::NAME.to_string()),
            factory_def,
            factory_args,
        )
        .await?;

    Ok(actor_ref)
}
