use async_graphql::{Context, Result, Subscription};
use futures::{Stream, StreamExt};
use tokio::sync::broadcast;

use crate::auth::AuthContext;
use crate::auth::scope::{Action, Domain, Resource, Scope};
use crate::device_registry::DeviceRegistry;
use crate::event_bus::{EventBus, EventBusMessage, EventFilter};

pub mod types;

use types::EventUpdate;

#[derive(Default)]
pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn events(
        &self,
        ctx: &Context<'_>,
        #[graphql(default_with = "\"*\".to_owned()")] filter: String,
    ) -> Result<impl Stream<Item = EventUpdate> + use<>> {
        let filter = EventFilter::parse(&filter).ok_or("invalid event filter")?;

        let auth = ctx.data::<AuthContext>()?;
        for kind in filter.domains() {
            let resource = Resource::for_event_kind(kind).ok_or("invalid event filter")?;
            if !auth.has(&Scope::new(Domain::Events, resource, Action::Read)) {
                return Err("insufficient scope".into());
            }
        }

        let rx = ctx.data::<EventBus>()?.subscribe();
        // Cheap Arc clone; moved into the stream so updates can resolve device
        // addresses to the same slug/name the `entities` query exposes.
        let registry = ctx.data::<DeviceRegistry>()?.clone();
        Ok(event_stream(rx, filter, registry))
    }
}

fn event_stream(
    rx: broadcast::Receiver<EventBusMessage>,
    filter: EventFilter,
    registry: DeviceRegistry,
) -> impl Stream<Item = EventUpdate> {
    futures::stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(msg) => return Some((msg, rx)),
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    })
    .filter_map(move |msg| {
        let keep = filter.matches(&msg);
        let registry = registry.clone();
        async move { keep.then(|| EventUpdate::from_message(msg, &registry)) }
    })
}
