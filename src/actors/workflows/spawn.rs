use super::{WorkflowWorker, WorkflowWorkerBuilder, WorkflowWorkerMessage};
use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, FactoryMessage, queues, routing},
};

pub async fn spawn_workflows(
    root_supervisor_ref: &ActorRef<()>,
) -> anyhow::Result<ActorRef<FactoryMessage<(), WorkflowWorkerMessage>>> {
    let door_handler_factory_def = Factory::<
        (),
        WorkflowWorkerMessage,
        (),
        WorkflowWorker,
        routing::QueuerRouting<(), WorkflowWorkerMessage>,
        queues::DefaultQueue<(), WorkflowWorkerMessage>,
    >::default();

    let door_handler_factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(WorkflowWorkerBuilder {}))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(5)
        .build();

    let (actor_ref, _) = root_supervisor_ref
        .spawn_linked(
            Some(WorkflowWorker::NAME.to_string()),
            door_handler_factory_def,
            door_handler_factory_args,
        )
        .await?;

    Ok(actor_ref)
}
