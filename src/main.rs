use home_gateway::actors::root::RootSupervisor;
use home_gateway::api::{build_router, build_schema};
use home_gateway::event_bus::EventBus;
use home_gateway::integrations::feature_flag::FeatureFlagClient;
use home_gateway::startup::{self, Handles, Tasks};
use home_gateway::state::{ApiState, AppState};
use home_gateway::tracing_setup::SampleRatios;
use home_gateway::utils::handle_cancellation;
use ractor::Actor;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let telemetry = startup::telemetry::init();

    let storage = startup::storage::init().await?;

    telemetry
        .sampling
        .replace(SampleRatios::from(&storage.settings.tracing.sampling));

    let feature_flag_client = FeatureFlagClient::new().await;

    let cancellation_token = CancellationToken::new();
    handle_cancellation(cancellation_token.clone());

    let Handles { registry, mqtt } =
        startup::handles::build(&storage, &feature_flag_client).await?;

    let listen_addr = storage.settings.http.listen_address;
    let devices = storage.devices.clone();

    let state = AppState {
        repos: storage.repos,
        settings: storage.settings,
        devices: storage.devices,
        db: storage.pool,
        feature_flag_client: feature_flag_client.clone(),
        sampling: telemetry.sampling,
        event_bus: EventBus::default(),
        handles: registry,
    };

    let schema = build_schema(&state);

    let (_, root_supervisor) = Actor::spawn(
        None,
        RootSupervisor {
            shared_actor_state: state.clone(),
        },
        (),
    )
    .await?;

    startup::api_keys::reconcile(&state).await;

    let event_bus = state.event_bus.clone();

    let router = build_router(
        ApiState {
            schema,
            inner: state,
        },
        telemetry.metrics_registry,
    );

    startup::tasks::run(
        listen_addr,
        Tasks {
            router,
            mqtt,
            devices,
            cancellation_token,
            feature_flag_client,
            event_bus,
            root_supervisor,
        },
    )
    .await
}
