use crate::state::SharedActorState;

use super::{Message, RobotVacuumHandler, RobotVacuumHandlerBuilder};
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, queues, routing},
};

pub async fn spawn_robot_vacuum_handler(
    root_supervisor_ref: &ActorRef<crate::actors::root::RootMessage>,
    shared_actor_state: SharedActorState,
) -> anyhow::Result<()> {
    let factory_def = Factory::<
        (),
        Message,
        (),
        RobotVacuumHandler,
        routing::QueuerRouting<(), Message>,
        queues::DefaultQueue<(), Message>,
    >::default();

    let factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(RobotVacuumHandlerBuilder { shared_actor_state }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(2)
        .build();

    let (_, _) = root_supervisor_ref
        .spawn_linked(
            Some(RobotVacuumHandler::NAME.to_string()),
            factory_def,
            factory_args,
        )
        .await?;

    Ok(())
}
