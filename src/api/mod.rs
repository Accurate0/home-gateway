use crate::actors::workflows::manager::WorkflowManager;
use crate::eink::manager::EinkDisplayManager;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::mqtt::MqttClient;
use crate::integrations::s3::S3;
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
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};

use crate::auth::auth_middleware;
use crate::device_registry::DeviceRegistry;
use crate::event_bus::EventBus;
use crate::graphql::{
    FinalSchema, QueryRoot,
    dataloader::device_battery::DeviceBatteryDataLoader,
    dataloader::device_battery_history::DeviceBatteryHistoryDataLoader,
    dataloader::eink_battery::EinkDisplayDataLoader,
    dataloader::forecast::ForecastDataLoader,
    dataloader::home_assistant_state::HomeAssistantStateDataLoader,
    dataloader::last_seen::LastSeenDataLoader,
    dataloader::media_player_state::MediaPlayerStateDataLoader,
    dataloader::plant::LatestPlantDataLoader,
    dataloader::robot_vacuum_state::RobotVacuumStateDataLoader,
    dataloader::temperature::LatestTemperatureDataLoader,
    dataloader::workflow_run_steps::WorkflowRunStepsDataLoader,
    handler::{graphiql, graphql_handler, graphql_ws_handler},
    mutations::MutationRoot,
    subscription::SubscriptionRoot,
};
use crate::integrations::feature_flag::FeatureFlagClient;
use crate::repo::RepoRegistry;
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
use crate::settings::SettingsContainer;
use crate::state::{AppState, HandleRegistry};
use sqlx::{Pool, Postgres};

const UNMATCHED_ROUTE: &str = "unmatched";

fn metric_route(req: &Request) -> String {
    req.extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|matched| matched.as_str().to_owned())
        .unwrap_or_else(|| UNMATCHED_ROUTE.to_owned())
}

async fn log_request(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    if path.contains("/health") || path.contains("/metrics") {
        return next.run(req).await;
    }

    let method = req.method().clone();
    let route = metric_route(&req);
    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let elapsed = start.elapsed();
    let status = response.status().as_u16();

    crate::metrics::record_rest_request(method.to_string(), route, status, elapsed);

    tracing::info!(
        %method,
        path = %path,
        status,
        elapsed_ms = elapsed.as_millis() as u64,
        "http request"
    );
    response
}

pub struct SchemaParts<'a> {
    pub db: &'a Pool<Postgres>,
    pub repos: &'a RepoRegistry,
    pub settings: &'a SettingsContainer,
    pub devices: &'a DeviceRegistry,
    pub event_bus: &'a EventBus,
    pub feature_flag_client: &'a FeatureFlagClient,
    pub handles: &'a HandleRegistry,
}

pub fn build_schema(state: &SchemaParts<'_>) -> FinalSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot,
    )
    .data(DataLoader::new(
        LatestTemperatureDataLoader {
            repo: state.repos.environment().clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        LatestPlantDataLoader {
            repo: state.repos.plant().clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        WorkflowRunStepsDataLoader {
            repo: state.repos.workflow().clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        LastSeenDataLoader {
            repo: state.repos.device().clone(),
            devices: state.devices.clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        EinkDisplayDataLoader {
            repo: state.repos.eink().clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        DeviceBatteryDataLoader {
            repo: state.repos.battery().clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        DeviceBatteryHistoryDataLoader {
            repo: state.repos.battery().clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        HomeAssistantStateDataLoader {
            repo: state.repos.home_assistant().clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        MediaPlayerStateDataLoader {
            repo: state.repos.media_player().clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        RobotVacuumStateDataLoader {
            repo: state.repos.robot_vacuum().clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        ForecastDataLoader {
            repo: state.repos.willyweather().clone(),
        },
        tokio::spawn,
    ))
    .data(state.handles.get::<HomeAssistant>().cloned())
    .data(state.handles.require::<EinkDisplayManager>().clone())
    .data(state.handles.require::<S3>().clone())
    .data(state.handles.clone())
    .data(state.handles.require::<MqttClient>().clone())
    .data(state.db.clone())
    .data(state.settings.clone())
    .data(state.devices.clone())
    .data(state.feature_flag_client.clone())
    .data(state.event_bus.clone())
    .data(state.handles.require::<WorkflowManager>().clone())
    .data(state.repos.clone())
    .extension(crate::graphql_tracing::Tracing)
    .limit_depth(state.settings.graphql.max_depth)
    .limit_complexity(state.settings.graphql.max_complexity)
    .finish()
}

pub fn build_router(state: AppState, metrics_registry: Registry) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(AllowHeaders::any());

    let scripted = routes::endpoints::mount(&state.settings.endpoints);

    let api_routes = scripted
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route("/schema", get(schema_route))
        .route("/control/light", post(light_control))
        .route("/workflow/execute", post(workflow_execute))
        .route("/lua/execute", post(routes::lua::execute::lua_execute))
        .route("/lua/repl", get(routes::lua::repl::lua_repl))
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
        .route_layer(from_fn_with_state(state.clone(), auth_middleware))
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
        .layer(from_fn(log_request))
        .with_state(state);

    Router::new().nest("/v1", api_routes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::endpoint::RESERVED_PREFIXES;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn the_metric_route_is_the_template_not_the_concrete_path() {
        let seen: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let captured = seen.clone();

        let inner = Router::new()
            .route("/epd/image/{hash}", get(|| async { "ok" }))
            .layer(from_fn(move |req: Request, next: Next| {
                let captured = captured.clone();
                async move {
                    *captured.lock().unwrap() = Some(metric_route(&req));
                    next.run(req).await
                }
            }));

        let app = Router::new().nest("/v1", inner);

        let request = Request::builder()
            .uri("/v1/epd/image/abc123")
            .body(axum::body::Body::empty())
            .unwrap();

        tower::ServiceExt::oneshot(app, request).await.unwrap();

        let route = seen.lock().unwrap().clone();

        assert_eq!(
            route.as_deref(),
            Some("/v1/epd/image/{hash}"),
            "the metric label must be the route template, or every hash becomes its own series"
        );
    }

    const BUILT_IN: &[&str] = &[
        "/graphql",
        "/graphql/ws",
        "/schema",
        "/control/light",
        "/workflow/execute",
        "/lua/execute",
        "/lua/repl",
        "/ingest/synergy",
        "/ingest/home/alarm",
        "/ingest/home/push-token",
        "/ingest/unifi",
        "/ingest/lua/{name}",
        "/epd/config",
        "/epd/image/{hash}",
        "/epd/firmware",
        "/epd/take-screenshot",
        "/push/notify",
        "/admin/keys",
        "/admin/keys/{id}",
        "/admin/keys/{id}/regenerate",
        "/weather/forecast",
        "/health",
        "/health/actors",
        "/metrics",
    ];

    #[test]
    fn every_built_in_route_is_covered_by_a_reserved_prefix() {
        for path in BUILT_IN {
            let covered = RESERVED_PREFIXES.iter().any(|prefix| {
                path == prefix
                    || path
                        .strip_prefix(prefix)
                        .is_some_and(|rest| rest.starts_with('/'))
            });

            assert!(
                covered,
                "`{path}` is not covered by RESERVED_PREFIXES, so config could shadow it"
            );
        }
    }
}
