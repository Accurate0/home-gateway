use std::time::{Duration, Instant};

pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Extension};
use reqwest_tracing::{
    DisableOtelPropagation, ReqwestOtelSpanBackend, TracingMiddleware, default_on_request_end,
    reqwest_otel_span,
};
use tracing::Span;

#[derive(Clone)]
pub struct UrlTemplate(pub &'static str);

const REDACTED: &str = "{redacted}";

fn placeholder_for(segment: &str) -> Option<&'static str> {
    if segment.is_empty() {
        return None;
    }

    if segment.chars().all(|c| c.is_ascii_digit()) {
        return Some("{id}");
    }

    let opaque = segment.len() >= 16
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    opaque.then_some(REDACTED)
}

pub fn redacted_path(url: &reqwest::Url) -> String {
    let path = url
        .path()
        .split('/')
        .map(|segment| placeholder_for(segment).unwrap_or(segment))
        .collect::<Vec<_>>()
        .join("/");

    if url.query().is_some() {
        format!("{path}?{REDACTED}")
    } else {
        path
    }
}

pub struct TimeTrace;
impl ReqwestOtelSpanBackend for TimeTrace {
    fn on_request_start(req: &Request, extension: &mut http::Extensions) -> Span {
        let host = req.url().host_str().unwrap_or("unknown");
        let template = extension
            .get::<UrlTemplate>()
            .map(|template| template.0.to_owned())
            .unwrap_or_else(|| redacted_path(req.url()));

        extension.insert(Instant::now());

        reqwest_otel_span!(
            name = format!("{} {}{}", req.method(), host, template),
            req,
            url.template = template,
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
            .timeout(REQUEST_TIMEOUT)
            .build()?,
    )
    .with(TracingMiddleware::<TimeTrace>::new())
    .build())
}
