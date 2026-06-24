use super::{Message, MqttIngest, MqttMessageHandlerBuilder};
use crate::types::SharedActorState;
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, FactoryMessage, queues, routing},
};

pub async fn spawn_mqtt_ingest(
    root_supervisor_ref: &ActorRef<()>,
    shared_actor_state: SharedActorState,
) -> anyhow::Result<ActorRef<FactoryMessage<(), Message>>> {
    let door_handler_factory_def = Factory::<
        (),
        Message,
        (),
        MqttIngest,
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
            Some(MqttIngest::NAME.to_string()),
            door_handler_factory_def,
            door_handler_factory_args,
        )
        .await?;

    Ok(actor_ref)
}
