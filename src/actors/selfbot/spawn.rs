use crate::{feature_flag::FeatureFlagClient, http::get_http_client, settings::Settings};
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, FactoryMessage, queues, routing},
};

use super::{SelfBotMessage, SelfBotWorker, SelfBotWorkerBuilder};

pub async fn spawn_selfbot(
    root_supervisor_ref: &ActorRef<()>,
    feature_flag_client: FeatureFlagClient,
    settings: Settings,
) -> anyhow::Result<ActorRef<FactoryMessage<(), SelfBotMessage>>> {
    let door_handler_factory_def = Factory::<
        (),
        SelfBotMessage,
        (),
        SelfBotWorker,
        routing::QueuerRouting<(), SelfBotMessage>,
        queues::DefaultQueue<(), SelfBotMessage>,
    >::default();

    let client = get_http_client()?;

    let door_handler_factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(SelfBotWorkerBuilder {
            client,
            settings,
            feature_flag_client,
        }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(1)
        .build();

    let (actor_ref, _) = root_supervisor_ref
        .spawn_linked(
            Some(SelfBotWorker::NAME.to_string()),
            door_handler_factory_def,
            door_handler_factory_args,
        )
        .await?;

    Ok(actor_ref)
}
