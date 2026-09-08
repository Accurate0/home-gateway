use ractor::{
    ActorRef,
    factory::{Factory, FactoryArguments, FactoryMessage, queues, routing},
};

use super::{ReconcilerMessage, ReconcilerWorker, ReconcilerWorkerBuilder};
use crate::state::AppState;

pub async fn spawn_reconciler(
    root_supervisor_ref: &ActorRef<crate::actors::root::RootMessage>,
    shared_actor_state: AppState,
) -> anyhow::Result<ActorRef<FactoryMessage<String, ReconcilerMessage>>> {
    let factory_def = Factory::<
        String,
        ReconcilerMessage,
        (),
        ReconcilerWorker,
        routing::StickyQueuerRouting<String, ReconcilerMessage>,
        queues::DefaultQueue<String, ReconcilerMessage>,
    >::default();

    let workers = shared_actor_state.settings.reconciler.workers;

    let factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(ReconcilerWorkerBuilder { shared_actor_state }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(workers)
        .build();

    let (actor_ref, _) = root_supervisor_ref
        .spawn_linked(
            Some(ReconcilerWorker::NAME.to_string()),
            factory_def,
            factory_args,
        )
        .await?;

    Ok(actor_ref)
}
