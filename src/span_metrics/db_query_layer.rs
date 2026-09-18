use tracing::{Event, Level, Subscriber, span};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

use super::query_timing::QueryTiming;

const DB_SPAN_PREFIX: &str = "db.";

pub struct DbQueryLayer;

impl<S> Layer<S> for DbQueryLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        if !attrs.metadata().name().starts_with(DB_SPAN_PREFIX) {
            return;
        }

        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(QueryTiming::start());
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        if *event.metadata().level() != Level::ERROR {
            return;
        }

        if let Some(span) = ctx.event_span(event)
            && let Some(timing) = span.extensions_mut().get_mut::<QueryTiming>()
        {
            timing.fail();
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(&id) else {
            return;
        };

        let extensions = span.extensions();

        let Some(timing) = extensions.get::<QueryTiming>() else {
            return;
        };

        crate::metrics::record_db_query(span.metadata().name(), timing.outcome(), timing.elapsed());
    }
}
