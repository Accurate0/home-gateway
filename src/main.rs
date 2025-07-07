use actors::{
    event_handler::{self},
    root::RootSupervisor,
};
use async_graphql::{EmptyMutation, EmptySubscription, Schema, dataloader::DataLoader};
use auth::RequireIpAuth;
use axum::{
    middleware::from_extractor_with_state,
    routing::{get, post},
};
use feature_flag::FeatureFlagClient;
use graphql::{
    QueryRoot,
    dataloader::temperature::LatestTemperatureDataLoader,
    handler::{graphiql, graphql_handler},
};
use http::Method;
use mqtt::Mqtt;
use ractor::{Actor, ActorRef, factory::FactoryMessage};
use routes::{health::health, ingest::maccas::maccas, schema::schema as schema_route};
use settings::Settings;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::{collections::HashSet, net::SocketAddr, sync::Arc};
use tokio::{sync::RwLock, task::JoinSet};
use tokio_util::sync::CancellationToken;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use tracing::Level;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};
use types::{ApiState, MainError, SharedActorState};
use unifi::Unifi;
use utils::{axum_shutdown_signal, handle_cancellation};

mod actors;
mod auth;
mod feature_flag;
mod graphql;
mod maccas;
mod mqtt;
mod routes;
mod settings;
mod timed_average;
mod timedelta_format;
mod types;
mod unifi;
mod utils;
mod zigbee2mqtt;

async fn init_actors(
    settings: Settings,
    db: Pool<Postgres>,
    known_devices_map: Arc<RwLock<HashSet<String>>>,
) -> anyhow::Result<ActorRef<FactoryMessage<(), event_handler::Message>>> {
    let shared_actor_state = SharedActorState {
        db,
        known_devices_map,
    };

    let (root_supervisor_ref, _) = Actor::spawn(
        None,
        RootSupervisor {
            shared_actor_state: shared_actor_state.clone(),
            settings,
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
    tracing_subscriber::registry()
        .with(
            Targets::default()
                .with_target("otel::tracing", Level::TRACE)
                .with_target("sea_orm::database", Level::TRACE)
                .with_default(Level::INFO),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = Settings::new()?;
    let pool = PgPoolOptions::new()
        .min_connections(0)
        .max_connections(20)
        .connect(&settings.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let feature_flag_client = FeatureFlagClient::new().await;

    let mut mqtt = Mqtt::new(settings.mqtt_url.clone(), 1883).await?;
    let unifi = Unifi::new(
        settings.unifi_api_key.clone(),
        settings.unifi_site_id.clone(),
        settings.unifi_api_base.clone(),
    )?;

    let cancellation_token = CancellationToken::new();
    handle_cancellation(cancellation_token.clone());

    let known_devices_map = Arc::new(RwLock::new(HashSet::new()));
    let event_handler_actor =
        init_actors(settings.clone(), pool.clone(), known_devices_map).await?;

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
        .allow_origin(AllowOrigin::list([
            "http://localhost:5173".parse()?,
            "https://home.anurag.sh".parse()?,
        ]))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(AllowHeaders::any());

    let api_state = ApiState {
        feature_flag_client,
        event_handler: event_handler_actor.clone(),
        schema,
        settings: settings.clone(),
        db: pool.clone(),
    };

    let api_routes = axum::Router::new()
        .route("/health", get(health))
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route("/schema", get(schema_route))
        .route_layer(from_extractor_with_state::<RequireIpAuth, ApiState>(
            api_state.clone(),
        ))
        .route("/ingest/maccas", post(maccas))
        .layer(cors)
        .with_state(api_state);
    let app = axum::Router::new().nest("/v1", api_routes);

    let addr = "[::]:8000".parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("starting api server {addr}");

    tokio::spawn(
        axum::serve(listener, app)
            .with_graceful_shutdown(axum_shutdown_signal())
            .into_future(),
    );

    let mut task_set = JoinSet::new();

    let mqtt_cancellation_token = cancellation_token.child_token();
    let mqtt_event_handler = event_handler_actor.clone();
    task_set.spawn(async move {
        mqtt.process_events(mqtt_cancellation_token, mqtt_event_handler)
            .await?;

        Ok::<(), MainError>(())
    });

    let unifi_cancellation_token = cancellation_token.child_token();
    let unifi_event_handler = event_handler_actor.clone();
    task_set.spawn(async move {
        unifi
            .process_events(unifi_cancellation_token, unifi_event_handler)
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
