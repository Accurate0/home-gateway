use crate::routes::{
    epd,
    ingest::{solar::solar, unifi::unifi},
    workflow::execute::workflow_execute,
};
use ::http::Method;
use actors::{
    event_handler::{self},
    reminder::{ReminderActor, ReminderActorDelayQueueValue, background::reminder_background},
    root::RootSupervisor,
};
use async_graphql::{EmptyMutation, EmptySubscription, Schema, dataloader::DataLoader};
use auth::RequireApiKey;
use axum::{
    body::Body,
    middleware::from_extractor_with_state,
    routing::{get, post},
};
use bucket::S3BucketAccessor;
use delayqueue::DelayQueue;
use discord::start_discord;
use feature_flag::FeatureFlagClient;
use graphql::{
    QueryRoot,
    dataloader::temperature::LatestTemperatureDataLoader,
    handler::{graphiql, graphql_handler},
};
use mqtt::{Mqtt, MqttClient};
use ractor::{Actor, ActorRef, factory::FactoryMessage};
use routes::{
    control::light::light_control,
    health::health,
    ingest::{home::alarm::alarm, maccas::maccas, synergy::synergy},
    schema::schema as schema_route,
};
use rustls::crypto::aws_lc_rs;
use settings::{IEEEAddress, SettingsContainer};
use sqlx::{
    ConnectOptions, Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinSet};
use tokio_util::sync::CancellationToken;
use tower_http::{
    LatencyUnit,
    cors::{AllowHeaders, AllowOrigin, CorsLayer},
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use types::{ApiState, MainError, SharedActorState};
use utils::{axum_shutdown_signal, handle_cancellation};

mod actors;
mod auth;
mod bucket;
mod delayqueue;
mod discord;
mod feature_flag;
mod graphql;
mod http;
mod mqtt;
mod notify;
mod routes;
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
    bucket_accessor: S3BucketAccessor,
    feature_flag_client: FeatureFlagClient,
    mqtt_client: MqttClient,
    db: Pool<Postgres>,
    known_devices_map: Arc<RwLock<HashMap<IEEEAddress, String>>>,
    reminder_delayqueue: DelayQueue<ReminderActorDelayQueueValue>,
) -> anyhow::Result<ActorRef<FactoryMessage<(), event_handler::Message>>> {
    let shared_actor_state = SharedActorState {
        settings,
        db,
        mqtt: mqtt_client,
        bucket_accessor,
        feature_flag_client,
        known_devices_map,
    };

    let (root_supervisor_ref, _) = Actor::spawn(
        None,
        RootSupervisor {
            shared_actor_state: shared_actor_state.clone(),
            reminder_delayqueue,
        },
        (),
    )
    .await?;

    let event_handler_actor =
        event_handler::spawn::spawn_event_handler(&root_supervisor_ref, shared_actor_state.clone())
            .await?;

    Ok(event_handler_actor)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    aws_lc_rs::default_provider().install_default().unwrap();
    tracing_setup::init()?;

    let settings_container = SettingsContainer::new()?;
    let settings = settings_container.load_full();

    let eink_display_bucket_credentials = s3::creds::Credentials::new(
        Some(&settings.aws_access_key_id),
        Some(&settings.aws_secret_access_key),
        None,
        None,
        None,
    )?;

    let eink_display_bucket = s3::Bucket::new(
        "home-gateway-image-bucket",
        s3::Region::ApSoutheast2,
        eink_display_bucket_credentials,
    )?;

    let bucket_accessor = S3BucketAccessor::new(eink_display_bucket);

    let pg_connect_options = PgConnectOptions::from_url(&settings.database_url.parse()?)?
        .log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(6));

    let pool = PgPoolOptions::new()
        .min_connections(0)
        .max_connections(20)
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

    let known_devices_map = Arc::new(RwLock::new(HashMap::new()));

    let reminder_delayqueue =
        DelayQueue::new(pool.clone(), ReminderActor::QUEUE_NAME.to_owned()).await?;

    let event_handler_actor = init_actors(
        settings_container.clone(),
        bucket_accessor.clone(),
        feature_flag_client.clone(),
        mqtt_client,
        pool.clone(),
        known_devices_map,
        reminder_delayqueue.clone(),
    )
    .await?;

    let schema = Schema::build(QueryRoot::default(), EmptyMutation, EmptySubscription)
        .data(DataLoader::new(
            LatestTemperatureDataLoader {
                database: pool.clone(),
            },
            tokio::spawn,
        ))
        .data(pool.clone())
        .finish();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(AllowHeaders::any());

    let object_registry_api_client = object_registry::ApiClient::new(
        settings.object_registry_private_key.clone(),
        settings.object_registry_key_id.clone(),
        "home-gateway/api",
    );

    let api_state = ApiState {
        feature_flag_client,
        object_registry: object_registry_api_client,
        event_handler: event_handler_actor.clone(),
        bucket_accessor,
        schema,
        settings: settings_container.clone(),
        db: pool.clone(),
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
        .route_layer(from_extractor_with_state::<RequireApiKey, ApiState>(
            api_state.clone(),
        ))
        .route("/health", get(health))
        .route("/ingest/home/alarm", post(alarm))
        .route("/ingest/maccas", post(maccas))
        .route("/ingest/unifi", post(unifi))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &::http::Request<Body>| {
                    tracing::info_span!("api", uri = request.uri().to_string())
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
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
    let mqtt_event_handler = event_handler_actor.clone();
    task_set.spawn(async move {
        mqtt.process_events(mqtt_cancellation_token, mqtt_event_handler)
            .await?;
        Ok::<(), MainError>(())
    });

    if !cfg!(debug_assertions) {
        let discord_cancellation_token = cancellation_token.child_token();
        task_set.spawn(async move {
            start_discord(
                settings.discord_token.clone(),
                pool.clone(),
                discord_cancellation_token,
            )
            .await?;
            Ok::<(), MainError>(())
        });
    }

    let reminder_cancellation_token = cancellation_token.child_token();
    task_set.spawn(async move {
        reminder_background(reminder_delayqueue, reminder_cancellation_token).await?;
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

// TODO:
// - store the built file to object registry via CI
// - use events to create a new screenshot if the file in object registry has changed
// - read the file from object registry instead of bucket
// - store the screenshot in object registry too i guess
