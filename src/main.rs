use actors::root::RootSupervisor;
use actors::workflows::manager::WorkflowManager;
use auth::{AuthManager, OAuthValidator};
use error::MainError;
use event_bus::EventBus;
use feature_flag::FeatureFlagClient;
use home_gateway::api::{build_router, build_schema};
use home_gateway::eink::EinkDisplayManager;
use home_gateway::integrations::solar::goodwe::GoodWeSemsAPI;
use home_gateway::{
    actors, auth, db, error, event_bus,
    integrations::{
        feature_flag, fuelwatch::FuelWatch, home_assistant, jellyfin, mqtt, s3,
        willyweather::WillyWeather,
    },
    settings, state, tracing_setup, utils,
};
use mqtt::Mqtt;
use ractor::Actor;
use rustls::crypto::aws_lc_rs;
use s3::S3;
use settings::SettingsContainer;
use state::{ApiState, AppState, HandleRegistry};
use std::net::SocketAddr;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use utils::{axum_shutdown_signal, handle_cancellation};

async fn init_actors(shared_actor_state: AppState) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    let (_, root_supervisor_handle) =
        Actor::spawn(None, RootSupervisor { shared_actor_state }, ()).await?;

    Ok(root_supervisor_handle)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    aws_lc_rs::default_provider().install_default().unwrap();
    tracing_setup::init();
    let metrics_registry = tracing_setup::init_metrics();

    let (settings_container, device_registry) = SettingsContainer::new()?;
    let settings = settings_container.clone();

    let pool = db::connect(&settings.database_url).await?;
    db::migrate(&pool).await?;

    let workflow_manager = WorkflowManager::new(pool.clone());

    let repos = home_gateway::repo::RepoRegistry::new(pool.clone());

    let feature_flag_client = FeatureFlagClient::new().await;

    let (mqtt_client, mut mqtt) = Mqtt::new(
        settings.mqtt_url.clone(),
        1883,
        settings.mqtt_username.clone(),
        settings.mqtt_password.clone(),
    )
    .await?;

    let cancellation_token = CancellationToken::new();
    handle_cancellation(cancellation_token.clone());

    let s3 = S3::new(
        &settings.s3.bucket,
        &settings.s3.region,
        settings.s3.endpoint.clone(),
    )?;

    let event_bus = EventBus::default();

    let eink = EinkDisplayManager::new(
        pool.clone(),
        s3.clone(),
        feature_flag_client.clone(),
        device_registry.clone(),
        settings_container.clone(),
        home_gateway::integrations::reddit::Reddit::new(),
    );

    let home_assistant =
        home_assistant::HomeAssistant::from_settings(&settings_container.home_assistant);

    let jellyfin = settings_container
        .jellyfin
        .as_ref()
        .and_then(jellyfin::Jellyfin::new);

    let transperth = settings_container
        .transperth
        .as_ref()
        .map(home_gateway::integrations::transperth::Transperth::new)
        .transpose()?;

    let http_client = home_gateway::http::get_traced_http_client()?;

    let actor_health = home_gateway::actors::health::ActorHealthRegistry::new();

    let willyweather = WillyWeather::new(&settings.willyweather)?;

    let fuelwatch = match settings.fuelwatch.as_ref() {
        Some(fuelwatch) => Some(FuelWatch::new(fuelwatch)?),
        None => None,
    };

    let goodwe = settings_container
        .solar
        .as_ref()
        .and_then(|solar| GoodWeSemsAPI::new(pool.clone(), solar));

    let oauth = match settings.oauth.clone() {
        Some(oauth_settings) => Some(std::sync::Arc::new(OAuthValidator::new(oauth_settings)?)),
        None => None,
    };

    let handles = HandleRegistry::builder()
        .insert(mqtt_client)
        .insert(s3)
        .insert(eink)
        .insert(workflow_manager)
        .insert(actor_health)
        .insert(AuthManager::new(pool.clone(), oauth))
        .insert(willyweather)
        .insert(http_client)
        .insert_optional(home_assistant)
        .insert_optional(jellyfin)
        .insert_optional(transperth)
        .insert_optional(fuelwatch)
        .insert_optional(goodwe)
        .build();

    assert_required_handles(&handles);

    let state = AppState {
        repos,
        settings: settings_container.clone(),
        devices: device_registry.clone(),
        db: pool.clone(),
        feature_flag_client,
        event_bus,
        handles,
    };

    let schema = build_schema(&state);

    let root_supervisor_handle = init_actors(state.clone()).await?;

    for key in &settings.api_keys {
        match state
            .handles
            .expect::<AuthManager>()
            .claim(&key.name, &key.scopes, key.expires_at)
            .await
        {
            Ok(true) => tracing::info!(name = %key.name, "reconciled api key scopes from config"),
            Ok(false) => tracing::warn!(
                name = %key.name,
                "config api key not found; mint it via the admin API"
            ),
            Err(e) => tracing::error!(name = %key.name, error = %e, "failed to reconcile api key"),
        }
    }

    let app = build_router(
        ApiState {
            schema,
            inner: state,
        },
        metrics_registry,
    );

    let addr = "[::]:8000".parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("starting api server {addr}");

    let mut task_set = JoinSet::new();

    task_set.spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(axum_shutdown_signal())
            .await
            .map_err(MainError::from)
    });

    let mqtt_cancellation_token = cancellation_token.child_token();
    let mqtt_devices = device_registry.clone();
    task_set.spawn(async move {
        mqtt.process_events(mqtt_cancellation_token, mqtt_devices)
            .await?;
        Ok::<(), MainError>(())
    });

    task_set.spawn(async move {
        root_supervisor_handle.await?;
        tracing::error!("the root supervisor stopped, shutting down");
        Ok::<(), MainError>(())
    });

    if let Some(r) = task_set.join_next().await {
        match r {
            Ok(r) => match r {
                Ok(_) => {
                    tracing::warn!("task ended without error")
                }
                Err(e) => tracing::error!("task ended with {e}"),
            },
            Err(e) => tracing::error!("join error: {e}"),
        }
    };

    tracing::info!("shutting down all tasks");
    task_set.shutdown().await;

    Ok(())
}

// TODO: query brightness and temperature states too if possible

fn assert_required_handles(handles: &HandleRegistry) {
    for (label, present) in [
        ("mqtt", handles.contains::<mqtt::MqttClient>()),
        ("s3", handles.contains::<S3>()),
        (
            "eink",
            handles.contains::<home_gateway::eink::EinkDisplayManager>(),
        ),
        ("workflows", handles.contains::<WorkflowManager>()),
        (
            "actor health",
            handles.contains::<home_gateway::actors::health::ActorHealthRegistry>(),
        ),
        ("auth", handles.contains::<AuthManager>()),
        ("willyweather", handles.contains::<WillyWeather>()),
        (
            "http client",
            handles.contains::<reqwest_middleware::ClientWithMiddleware>(),
        ),
    ] {
        if !present {
            panic!("the {label} handle was not registered before startup");
        }
    }
}
