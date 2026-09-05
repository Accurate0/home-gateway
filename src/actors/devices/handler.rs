use crate::actors::root::RootMessage;
use crate::state::AppState;
use ractor::{
    ActorProcessingErr, ActorRef,
    factory::{
        Factory, FactoryArguments, FactoryMessage, Job, Worker, WorkerBuilder, WorkerId, queues,
        routing,
    },
};

pub const CONSECUTIVE_FAILURE_LIMIT: u32 = 5;

pub trait DeviceHandler: Send + Sync + Sized + 'static {
    const NAME: &'static str;
    const WORKERS: usize = 1;

    type Message: ractor::Message;
    type State: ractor::State + Default;

    fn new(shared_actor_state: AppState) -> Self;

    fn handle(
        &self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub struct HandlerState<T: DeviceHandler> {
    inner: T::State,
    consecutive_failures: u32,
}

impl<T: DeviceHandler> Default for HandlerState<T> {
    fn default() -> Self {
        Self {
            inner: T::State::default(),
            consecutive_failures: 0,
        }
    }
}

pub struct HandlerWorker<T: DeviceHandler>(T);

impl<T: DeviceHandler> Worker for HandlerWorker<T> {
    type Key = ();
    type Message = T::Message;
    type State = HandlerState<T>;
    type Arguments = ();

    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), T::Message>>,
        _startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(HandlerState::default())
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), T::Message>>,
        Job { msg, .. }: Job<(), T::Message>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match self.0.handle(msg, &mut state.inner).await {
            Ok(()) => {
                state.consecutive_failures = 0;

                Ok(())
            }
            Err(e) => {
                state.consecutive_failures += 1;
                crate::metrics::record_device_handler_error(T::NAME);

                if state.consecutive_failures >= CONSECUTIVE_FAILURE_LIMIT {
                    tracing::error!(
                        handler = T::NAME,
                        failures = state.consecutive_failures,
                        "handler failed repeatedly, failing worker: {e}"
                    );

                    return Err(e.into());
                }

                tracing::error!(
                    handler = T::NAME,
                    failures = state.consecutive_failures,
                    "error while handling message: {e}"
                );

                Ok(())
            }
        }
    }
}

pub struct HandlerBuilder<T: DeviceHandler> {
    shared_actor_state: AppState,
    _handler: std::marker::PhantomData<fn() -> T>,
}

impl<T: DeviceHandler> WorkerBuilder<HandlerWorker<T>, ()> for HandlerBuilder<T> {
    fn build(&mut self, _wid: usize) -> (HandlerWorker<T>, ()) {
        (HandlerWorker(T::new(self.shared_actor_state.clone())), ())
    }
}

pub async fn spawn_handler<T: DeviceHandler>(
    root_supervisor_ref: &ActorRef<RootMessage>,
    shared_actor_state: AppState,
) -> Result<ActorRef<FactoryMessage<(), T::Message>>, ActorProcessingErr> {
    let factory_def = Factory::<
        (),
        T::Message,
        (),
        HandlerWorker<T>,
        routing::QueuerRouting<(), T::Message>,
        queues::DefaultQueue<(), T::Message>,
    >::default();

    let factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(HandlerBuilder::<T> {
            shared_actor_state,
            _handler: std::marker::PhantomData,
        }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(T::WORKERS)
        .build();

    let (actor_ref, _) = root_supervisor_ref
        .spawn_linked(Some(T::NAME.to_string()), factory_def, factory_args)
        .await?;

    Ok(actor_ref)
}
