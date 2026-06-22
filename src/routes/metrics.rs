use axum::response::{IntoResponse, Response};
use http::{StatusCode, header::CONTENT_TYPE};
use prometheus::{Registry, TextEncoder};

/// Renders the Prometheus registry in the text exposition format for scraping.
pub fn render(registry: &Registry) -> Response {
    let metric_families = registry.gather();
    match TextEncoder::new().encode_to_string(&metric_families) {
        Ok(body) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/plain; version=0.0.4")],
            body,
        )
            .into_response(),
        Err(e) => {
            tracing::error!("failed to encode prometheus metrics: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
