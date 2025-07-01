use super::{EventHandler, Message, MqttMessageHandlerBuilder};
use crate::types::SharedActorState;
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, FactoryMessage, queues, routing},
};

pub async fn spawn_event_handler(
    root_supervisor_ref: &ActorRef<()>,
    shared_actor_state: SharedActorState,
) -> anyhow::Result<ActorRef<FactoryMessage<(), Message>>> {
    let door_handler_factory_def = Factory::<
        (),
        Message,
        (),
        EventHandler,
        routing::QueuerRouting<(), Message>,
        queues::DefaultQueue<(), Message>,
    >::default();

    let door_handler_factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(MqttMessageHandlerBuilder { shared_actor_state }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(5)
        .build();

    let (actor_ref, _) = root_supervisor_ref
        .spawn_linked(
            Some(EventHandler::NAME.to_string()),
            door_handler_factory_def,
            door_handler_factory_args,
        )
        .await?;

    Ok(actor_ref)
}
