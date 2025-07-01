use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, queues, routing},
};

use crate::types::SharedActorState;

use super::{LightHandler, LightHandlerBuilder, Message};

pub async fn spawn_light_handler(
    root_supervisor_ref: &ActorRef<()>,
    shared_actor_state: SharedActorState,
) -> anyhow::Result<()> {
    let door_handler_factory_def = Factory::<
        (),
        Message,
        (),
        LightHandler,
        routing::QueuerRouting<(), Message>,
        queues::DefaultQueue<(), Message>,
    >::default();

    let door_handler_factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(LightHandlerBuilder { shared_actor_state }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(1)
        .build();

    let (_, _) = root_supervisor_ref
        .spawn_linked(
            Some(LightHandler::NAME.to_string()),
            door_handler_factory_def,
            door_handler_factory_args,
        )
        .await?;

    Ok(())
}
