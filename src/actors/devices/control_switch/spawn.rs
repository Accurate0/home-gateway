use super::{ControlSwitchHandler, ControlSwitchHandlerBuilder, ControlSwitchMessage};
use crate::types::SharedActorState;
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, queues, routing},
};

pub async fn spawn_control_switch_handler(
    root_supervisor_ref: &ActorRef<()>,
    shared_actor_state: SharedActorState,
) -> anyhow::Result<()> {
    let door_handler_factory_def = Factory::<
        (),
        ControlSwitchMessage,
        (),
        ControlSwitchHandler,
        routing::QueuerRouting<(), ControlSwitchMessage>,
        queues::DefaultQueue<(), ControlSwitchMessage>,
    >::default();

    let door_handler_factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(ControlSwitchHandlerBuilder { shared_actor_state }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(3)
        .build();

    let (_, _) = root_supervisor_ref
        .spawn_linked(
            Some(ControlSwitchHandler::NAME.to_string()),
            door_handler_factory_def,
            door_handler_factory_args,
        )
        .await?;

    Ok(())
}
