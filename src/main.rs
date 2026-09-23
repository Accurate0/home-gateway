use home_gateway::actors::root::RootSupervisor;
use home_gateway::api::{SchemaParts, build_router, build_schema};
use home_gateway::eink::EinkDisplayManager;
use home_gateway::event_bus::EventBus;
use home_gateway::integrations::feature_flag::FeatureFlagClient;
use home_gateway::integrations::home_assistant::HomeAssistant;
use home_gateway::startup::{self, Handles, Tasks};
use home_gateway::state::AppState;
use home_gateway::tracing_setup::SampleRatios;
use home_gateway::utils::handle_cancellation;
use ractor::Actor;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let started = Instant::now();

    let telemetry = startup::telemetry::init();

    let storage = startup::storage::init().await?;

    telemetry
        .sampling
        .replace(SampleRatios::from(&storage.settings.tracing.sampling));

    let feature_flag_client = FeatureFlagClient::new().await;

    let cancellation_token = CancellationToken::new();
    handle_cancellation(cancellation_token.clone());

    let Handles {
        registry,
        mqtt,
        esphome_nodes,
    } = startup::handles::build(&storage, &feature_flag_client).await?;

    let listen_addr = storage.settings.http.listen_address;
    let devices = storage.devices.clone();
    let lua = startup::lua::build(&storage.settings, &registry)?;

    let event_bus = EventBus::default();

    let schema = build_schema(&SchemaParts {
        db: &storage.pool,
        repos: &storage.repos,
        settings: &storage.settings,
        devices: &storage.devices,
        event_bus: &event_bus,
        feature_flag_client: &feature_flag_client,
        handles: &registry,
    });

    let state = AppState {
        repos: storage.repos,
        settings: storage.settings,
        devices: storage.devices,
        db: storage.pool,
        feature_flag_client: feature_flag_client.clone(),
        sampling: telemetry.sampling,
        event_bus,
        handles: registry,
        lua,
        schema,
    };

    let (root_supervisor, root_supervisor_handle) = Actor::spawn(
        None,
        RootSupervisor {
            shared_actor_state: state.clone(),
        },
        (),
    )
    .await?;

    startup::api_keys::reconcile(&state).await;

    let event_bus = state.event_bus.clone();
    let home_assistant = state.handles.get::<HomeAssistant>().cloned();
    let home_assistant_websocket = state.settings.home_assistant.websocket;
    let esphome = state.settings.integrations.esphome.clone();

    let eink = state.handles.expect::<EinkDisplayManager>().clone();

    let router = build_router(state, telemetry.metrics_registry);

    startup::tasks::run(
        listen_addr,
        Tasks {
            router,
            mqtt,
            home_assistant,
            home_assistant_websocket,
            esphome,
            esphome_nodes,
            devices,
            cancellation_token,
            feature_flag_client,
            event_bus,
            root_supervisor,
            root_supervisor_handle,
            eink,
            started,
        },
    )
    .await
}
