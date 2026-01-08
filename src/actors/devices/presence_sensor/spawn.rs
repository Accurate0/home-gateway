use super::{Message, PresenceSensorHandler, PresenceSensorHandlerBuilder};
use crate::types::SharedActorState;
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, queues, routing},
};

pub async fn spawn_presence_handler(
    root_supervisor_ref: &ActorRef<()>,
    shared_actor_state: SharedActorState,
) -> anyhow::Result<()> {
    let door_handler_factory_def = Factory::<
        (),
        Message,
        (),
        PresenceSensorHandler,
        routing::QueuerRouting<(), Message>,
        queues::DefaultQueue<(), Message>,
    >::default();

    let door_handler_factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(PresenceSensorHandlerBuilder {
            shared_actor_state,
        }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(1)
        .build();

    let (_, _) = root_supervisor_ref
        .spawn_linked(
            Some(PresenceSensorHandler::NAME.to_string()),
            door_handler_factory_def,
            door_handler_factory_args,
        )
        .await?;

    Ok(())
}
