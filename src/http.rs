use std::time::{Duration, Instant};

use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Extension};
use reqwest_tracing::{
    DisableOtelPropagation, ReqwestOtelSpanBackend, TracingMiddleware, default_on_request_end,
    reqwest_otel_span,
};
use tracing::Span;

pub struct TimeTrace;
impl ReqwestOtelSpanBackend for TimeTrace {
    fn on_request_start(req: &Request, extension: &mut http::Extensions) -> Span {
        let url = req.url().as_str();
        extension.insert(Instant::now());

        reqwest_otel_span!(
            name = format!("{} {}", req.method(), url),
            req,
            url = url,
            time_elapsed = tracing::field::Empty,
            time_elapsed_formatted = tracing::field::Empty
        )
    }

    fn on_request_end(
        span: &Span,
        outcome: &reqwest_middleware::Result<Response>,
        extension: &mut http::Extensions,
    ) {
        let time_elapsed = extension.get::<Instant>().unwrap().elapsed().as_millis() as i64;
        default_on_request_end(span, outcome);
        span.record("time_elapsed", time_elapsed);
        span.record("time_elapsed_formatted", format!("{time_elapsed}ms"));
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HttpCreationError {
    #[error("Request builder error has occurred: `{0}`")]
    ReqwestBuilderError(#[from] reqwest::Error),
}

pub fn wrap_client_in_middleware_no_tracing(
    client: reqwest::Client,
) -> Result<ClientWithMiddleware, HttpCreationError> {
    Ok(ClientBuilder::new(client)
        .with_init(Extension(DisableOtelPropagation))
        .with(TracingMiddleware::<TimeTrace>::new())
        .build())
}

pub fn get_traced_http_client() -> Result<ClientWithMiddleware, HttpCreationError> {
    Ok(ClientBuilder::new(
        reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()?,
    )
    .with(TracingMiddleware::<TimeTrace>::new())
    .build())
}
