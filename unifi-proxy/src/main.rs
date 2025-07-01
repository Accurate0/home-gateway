use anyhow::Context;
use axum::{
    Router,
    extract::{DefaultBodyLimit, Path, Query, State},
    response::Response,
    routing::get,
};
use axum_shutdown::axum_shutdown_signal;
use futures::StreamExt;
use http::{HeaderMap, Method, Request, StatusCode};
use std::{collections::HashMap, net::SocketAddr, time::Duration};
use tower_http::{
    LatencyUnit,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use types::{AppError, AppState};

mod axum_shutdown;
mod types;

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn route(
    State(AppState {
        client,
        unifi_api_base,
    }): State<AppState>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    let url = format!("{unifi_api_base}/{path}");
    let unifi_response = client
        .request(method, url)
        .query(&params)
        .headers(headers)
        .send()
        .await?;

    let mut response = Response::builder().status(unifi_response.status());

    let headers_map = response.headers_mut().unwrap();
    for (header_name, header_value) in unifi_response.headers() {
        headers_map.insert(header_name.clone(), header_value.clone());
    }

    let stream = unifi_response
        .bytes_stream()
        .map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));

    let body = axum::body::Body::from_stream(stream);

    Ok(response.body(body)?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let unifi_api_base = std::env::var("UNIFI_API_BASE").context("must have unifi api base")?;
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let matched_path = request
                .extensions()
                .get::<Path<String>>()
                .map(|p| p.as_str())
                .unwrap_or_else(|| request.uri().path());

            tracing::info_span!("request", uri = matched_path)
        })
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Millis),
        );

    let router = Router::new()
        .route("/health", get(health))
        .route("/{*path}", get(route).post(route))
        .with_state(AppState {
            unifi_api_base,
            client: reqwest::ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .timeout(Duration::from_secs(60))
                .build()?,
        })
        .layer(DefaultBodyLimit::disable())
        .layer(trace_layer);

    let addr = "[::]:8001".parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("starting api server {addr}");
    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(axum_shutdown_signal())
        .await?;

    Ok(())
}
