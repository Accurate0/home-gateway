use crate::settings::Settings;
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, FactoryMessage, queues, routing},
};
use std::time::Duration;

use super::{SelfBotMessage, SelfBotWorker, SelfBotWorkerBuilder};

pub async fn spawn_selfbot(
    root_supervisor_ref: &ActorRef<()>,
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

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()?;

    let door_handler_factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(SelfBotWorkerBuilder { client, settings }))
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
