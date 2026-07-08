use crate::{
    graphql::subscription::SubscriptionRoot,
    routes::{
        epd,
        ingest::{solar::solar, unifi::unifi},
        workflow::execute::workflow_execute,
    },
};
use ::http::Method;
use actors::{
    mqtt_ingest::{self},
    root::RootSupervisor,
};
use async_graphql::{Schema, dataloader::DataLoader};
use auth::{AuthManager, OAuthValidator, auth_middleware};
use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post},
};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use device_registry::DeviceRegistry;
use event_bus::EventBus;
use feature_flag::FeatureFlagClient;
use graphql::{
    QueryRoot,
    dataloader::temperature::LatestTemperatureDataLoader,
    mutations::MutationRoot,
    handler::{graphiql, graphql_handler, graphql_ws_handler},
};
use mqtt::{Mqtt, MqttClient};
use ractor::{Actor, ActorRef, factory::FactoryMessage};
use routes::{
    admin::keys::{create_key, list_keys, revoke_key, update_key},
    control::light::light_control,
    health::{actor_health, health},
    ingest::{
        home::{alarm::alarm, push_token::push_token},
        synergy::synergy,
    },
    push::notify as push_notify,
    schema::schema as schema_route,
};
use rustls::crypto::aws_lc_rs;
use s3::S3;
use settings::SettingsContainer;
use sqlx::{
    ConnectOptions, Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::{net::SocketAddr, time::Duration};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use types::{ApiState, MainError, SharedActorState};
use utils::{axum_shutdown_signal, handle_cancellation};

mod actors;
mod auth;
mod device_registry;
mod esphome;
mod event_bus;
mod feature_flag;
mod graphql;
mod graphql_tracing;
mod http;
mod metrics;
mod mqtt;
mod notify;
mod routes;
mod s3;
mod settings;
mod timed_average;
mod timedelta_format;
mod timer;
mod tracing_setup;
mod tracker;
mod types;
mod utils;
mod woolworths;
mod zigbee2mqtt;

async fn init_actors(
    settings: SettingsContainer,
    devices: DeviceRegistry,
    feature_flag_client: FeatureFlagClient,
    mqtt_client: MqttClient,
    db: Pool<Postgres>,
    s3: S3,
    event_bus: EventBus,
) -> anyhow::Result<ActorRef<FactoryMessage<(), mqtt_ingest::Message>>> {
    let shared_actor_state = SharedActorState {
        settings,
        devices,
        db,
        mqtt: mqtt_client,
        feature_flag_client,
        s3,
        event_bus,
    };

    let (root_supervisor_ref, _) = Actor::spawn(
        None,
        RootSupervisor {
            shared_actor_state: shared_actor_state.clone(),
        },
        (),
    )
    .await?;

    let mqtt_ingest_actor =
        mqtt_ingest::spawn::spawn_mqtt_ingest(&root_supervisor_ref, shared_actor_state.clone())
            .await?;

    Ok(mqtt_ingest_actor)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    aws_lc_rs::default_provider().install_default().unwrap();
    tracing_setup::init();
    let metrics_registry = tracing_setup::init_metrics();

    let (settings_container, device_registry) = SettingsContainer::new()?;
    let settings = settings_container.clone();

    let pg_connect_options = PgConnectOptions::from_url(&settings.database_url.parse()?)?
        .log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(6));

    let pool = PgPoolOptions::new()
        .min_connections(0)
        .connect_with(pg_connect_options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

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

    let mqtt_ingest_actor = init_actors(
        settings_container.clone(),
        device_registry.clone(),
        feature_flag_client.clone(),
        mqtt_client,
        pool.clone(),
        s3.clone(),
        event_bus.clone(),
    )
    .await?;

    let schema = Schema::build(QueryRoot::default(), MutationRoot::default(), SubscriptionRoot)
        .data(DataLoader::new(
            LatestTemperatureDataLoader {
                database: pool.clone(),
            },
            tokio::spawn,
        ))
        .data(pool.clone())
        .data(settings_container.clone())
        .data(device_registry.clone())
        .data(event_bus)
        .extension(crate::graphql_tracing::Tracing)
        .finish();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(AllowHeaders::any());

    let oauth = match settings.oauth.clone() {
        Some(oauth_settings) => Some(std::sync::Arc::new(OAuthValidator::new(oauth_settings)?)),
        None => None,
    };

    let api_state = ApiState {
        feature_flag_client,
        schema,
        settings: settings_container.clone(),
        db: pool.clone(),
        s3,
        auth: AuthManager::new(pool.clone(), oauth),
    };

    let api_routes = axum::Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route("/schema", get(schema_route))
        .route("/control/light", post(light_control))
        .route("/workflow/execute", post(workflow_execute))
        .route("/ingest/synergy", post(synergy))
        .route("/ingest/solar", post(solar))
        .route("/epd/config", get(epd::config))
        .route("/epd/latest", get(epd::latest))
        .route("/epd/take-screenshot", post(epd::take_screenshot))
        .route("/push/notify", post(push_notify))
        .route("/ingest/home/alarm", post(alarm))
        .route("/ingest/home/push-token", post(push_token))
        .route("/ingest/unifi", post(unifi))
        .route("/admin/keys", post(create_key).get(list_keys))
        .route("/admin/keys/{id}", delete(revoke_key).patch(update_key))
        .route_layer(from_fn_with_state(api_state.clone(), auth_middleware))
        .layer(OtelAxumLayer::default())
        .route("/graphql/ws", get(graphql_ws_handler))
        .route("/health", get(health))
        .route("/health/actors", get(actor_health))
        .route(
            "/metrics",
            get(move || {
                let registry = metrics_registry.clone();
                async move { routes::metrics::render(&registry) }
            }),
        )
        .layer(cors)
        .with_state(api_state);

    let app = axum::Router::new().nest("/v1", api_routes);

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
    let mqtt_ingest = mqtt_ingest_actor.clone();
    task_set.spawn(async move {
        mqtt.process_events(mqtt_cancellation_token, mqtt_ingest)
            .await?;
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
