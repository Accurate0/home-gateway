use crate::state::SharedActorState;

use super::{MediaPlayerHandler, MediaPlayerHandlerBuilder, Message};
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, queues, routing},
};

pub async fn spawn_media_player_handler(
    root_supervisor_ref: &ActorRef<()>,
    shared_actor_state: SharedActorState,
) -> anyhow::Result<()> {
    let factory_def = Factory::<
        (),
        Message,
        (),
        MediaPlayerHandler,
        routing::QueuerRouting<(), Message>,
        queues::DefaultQueue<(), Message>,
    >::default();

    let factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(MediaPlayerHandlerBuilder { shared_actor_state }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(1)
        .build();

    let (_, _) = root_supervisor_ref
        .spawn_linked(
            Some(MediaPlayerHandler::NAME.to_string()),
            factory_def,
            factory_args,
        )
        .await?;

    Ok(())
}
