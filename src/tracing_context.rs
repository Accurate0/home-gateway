use std::collections::HashMap;

use opentelemetry::global;
use opentelemetry::propagation::{Extractor, Injector};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub type TraceParent = Option<String>;

const TRACEPARENT: &str = "traceparent";

#[derive(Default)]
struct Carrier(HashMap<String, String>);

impl Injector for Carrier {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_owned(), value);
    }
}

impl Extractor for Carrier {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(String::as_str).collect()
    }
}

pub fn inject_current() -> TraceParent {
    let context = Span::current().context();

    let mut carrier = Carrier::default();
    global::get_text_map_propagator(|propagator| propagator.inject_context(&context, &mut carrier));

    carrier.0.remove(TRACEPARENT)
}

pub fn context_of(traceparent: Option<&str>) -> Option<opentelemetry::Context> {
    let traceparent = traceparent?;

    let mut carrier = Carrier::default();
    carrier
        .0
        .insert(TRACEPARENT.to_owned(), traceparent.to_owned());

    let context = global::get_text_map_propagator(|propagator| propagator.extract(&carrier));

    Some(context)
}

pub fn set_parent(span: &Span, traceparent: Option<&str>) {
    if let Some(context) = context_of(traceparent)
        && let Err(e) = span.set_parent(context)
    {
        tracing::debug!("could not set span parent: {e}");
    }
}

pub fn add_link(span: &Span, traceparent: Option<&str>) {
    if let Some(context) = context_of(traceparent) {
        let linked = {
            use opentelemetry::trace::TraceContextExt;

            context.span().span_context().clone()
        };

        if linked.is_valid() {
            span.add_link(linked);
        }
    }
}

pub trait TracedMessage {
    fn traceparent(&self) -> Option<&str> {
        None
    }

    fn subject(&self) -> Option<&str> {
        None
    }
}

pub fn record_error(span: &Span, message: &str) {
    span.record("otel.status_code", "ERROR");
    span.record("otel.status_message", message);
}

pub fn record_current_error(message: &str) {
    record_error(&Span::current(), message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_absent_traceparent_yields_no_context() {
        assert!(context_of(None).is_none());
    }

    #[test]
    fn a_malformed_traceparent_does_not_panic() {
        assert!(context_of(Some("not-a-traceparent")).is_some());
    }
}
