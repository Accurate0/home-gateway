use async_graphql::{Schema, dataloader::DataLoader};
use axum::{
    Router,
    extract::Request,
    middleware::{Next, from_fn, from_fn_with_state},
    response::Response,
    routing::{delete, get, post},
};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use http::Method;
use prometheus::Registry;
use reqwest_middleware::ClientWithMiddleware;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};

use crate::auth::auth_middleware;
use crate::graphql::{
    FinalSchema, QueryRoot,
    dataloader::device_battery::DeviceBatteryDataLoader,
    dataloader::device_battery_history::DeviceBatteryHistoryDataLoader,
    dataloader::eink_battery::EinkDisplayDataLoader,
    dataloader::home_assistant_state::HomeAssistantStateDataLoader,
    dataloader::last_seen::LastSeenDataLoader,
    dataloader::media_player_state::MediaPlayerStateDataLoader,
    dataloader::robot_vacuum_state::RobotVacuumStateDataLoader,
    dataloader::temperature::LatestTemperatureDataLoader,
    handler::{graphiql, graphql_handler, graphql_ws_handler},
    mutations::MutationRoot,
    subscription::SubscriptionRoot,
};
use crate::integrations::fuelwatch::FuelWatch;
use crate::integrations::transperth::Transperth;
use crate::integrations::willyweather::WillyWeather;
use crate::routes::{
    self,
    admin::keys::{create_key, list_keys, regenerate_key, revoke_key, update_key},
    control::light::light_control,
    epd,
    health::{actor_health, health},
    ingest::{
        home::{alarm::alarm, push_token::push_token},
        synergy::synergy,
        unifi::unifi,
    },
    push::notify as push_notify,
    schema::schema as schema_route,
    workflow::execute::workflow_execute,
};
use crate::state::{ApiState, SharedActorState};

const GRAPHQL_MAX_DEPTH: usize = 20;
const GRAPHQL_MAX_COMPLEXITY: usize = 5000;

async fn log_request(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    if path.contains("/health") {
        return next.run(req).await;
    }

    let method = req.method().clone();
    let start = std::time::Instant::now();
    let response = next.run(req).await;
    tracing::info!(
        %method,
        path = %path,
        status = response.status().as_u16(),
        elapsed_ms = start.elapsed().as_millis() as u64,
        "http request"
    );
    response
}

pub fn build_schema(
    state: &SharedActorState,
    http_client: ClientWithMiddleware,
    willyweather: WillyWeather,
    transperth: Option<Transperth>,
    fuelwatch: Option<FuelWatch>,
) -> FinalSchema {
    let db = &state.db;

    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot,
    )
    .data(DataLoader::new(
        LatestTemperatureDataLoader {
            database: db.clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        LastSeenDataLoader {
            database: db.clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        EinkDisplayDataLoader {
            database: db.clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        DeviceBatteryDataLoader {
            database: db.clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        DeviceBatteryHistoryDataLoader {
            database: db.clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        HomeAssistantStateDataLoader {
            database: db.clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        MediaPlayerStateDataLoader {
            database: db.clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        RobotVacuumStateDataLoader {
            database: db.clone(),
        },
        tokio::spawn,
    ))
    .data(state.home_assistant.clone())
    .data(state.eink.clone())
    .data(state.s3.clone())
    .data(http_client)
    .data(willyweather)
    .data(transperth)
    .data(fuelwatch)
    .data(state.mqtt.clone())
    .data(state.db.clone())
    .data(state.settings.clone())
    .data(state.devices.clone())
    .data(state.feature_flag_client.clone())
    .data(state.event_bus.clone())
    .data(state.workflows.clone())
    .extension(crate::graphql_tracing::Tracing)
    .limit_depth(GRAPHQL_MAX_DEPTH)
    .limit_complexity(GRAPHQL_MAX_COMPLEXITY)
    .finish()
}

pub fn build_router(api_state: ApiState, metrics_registry: Registry) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(AllowHeaders::any());

    let api_routes = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route("/schema", get(schema_route))
        .route("/control/light", post(light_control))
        .route("/workflow/execute", post(workflow_execute))
        .route("/ingest/synergy", post(synergy))
        .route("/epd/config", post(epd::config))
        .route("/epd/image/{hash}", get(epd::image))
        .route("/epd/firmware", get(epd::firmware))
        .route("/epd/take-screenshot", post(epd::take_screenshot))
        .route("/push/notify", post(push_notify))
        .route("/ingest/home/alarm", post(alarm))
        .route("/ingest/home/push-token", post(push_token))
        .route("/ingest/unifi", post(unifi))
        .route("/admin/keys", post(create_key).get(list_keys))
        .route("/admin/keys/{id}", delete(revoke_key).patch(update_key))
        .route("/admin/keys/{id}/regenerate", post(regenerate_key))
        .route("/weather/forecast", get(routes::weather::forecast))
        .route_layer(from_fn_with_state(api_state.clone(), auth_middleware))
        .layer(OtelAxumLayer::default())
        .route("/solar/current", get(routes::solar::current))
        .route("/solar/history", get(routes::solar::history))
        .route("/solar/history/range", get(routes::solar::history_since))
        .route("/solar/health", get(routes::solar::health))
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
        .layer(from_fn(log_request))
        .with_state(api_state);

    Router::new().nest("/v1", api_routes)
}
