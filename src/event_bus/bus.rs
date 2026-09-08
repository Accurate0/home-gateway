use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use tokio::sync::broadcast;

use super::message::EventBusMessage;
use super::subscriber::{EventSubscriber, Outcome, Recipient, Route, Subscription, route};

/// Clonable handle to the in-memory event bus. Cheap to clone (shares one
/// broadcast sender and one route table). Stored on `AppState` so any actor can
/// publish.
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<EventBusMessage>,
    routes: Arc<RwLock<Vec<Route>>>,
    next_route_id: Arc<AtomicU64>,
}

impl EventBus {
    /// `capacity` bounds the per-subscriber backlog; slow subscribers that fall
    /// behind observe a `Lagged` error rather than blocking producers.
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self {
            tx,
            routes: Arc::new(RwLock::new(Vec::new())),
            next_route_id: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Forward matching events straight into an actor's mailbox. The returned
    /// [`Subscription`] deregisters on drop, so an actor keeps it in its state
    /// and a stop or supervisor restart needs no explicit teardown.
    pub fn register<S: EventSubscriber>(
        &self,
        label: &'static str,
        recipient: Recipient<S::Msg>,
        subscriber: S,
    ) -> Subscription {
        let id = self.next_route_id.fetch_add(1, Ordering::Relaxed);

        match self.routes.write() {
            Ok(mut routes) => {
                routes.push(route(id, label, recipient, subscriber));
                tracing::info!("{label} subscribed to {:?}", S::KINDS);
            }
            Err(e) => tracing::error!("could not register {label} on the event bus: {e}"),
        }

        Subscription::new(id, &self.routes)
    }

    /// Publish an event: forward it to every registered subscriber whose kinds
    /// match, then fan it out on the broadcast channel for stream consumers such
    /// as the GraphQL subscription.
    pub fn publish(&self, msg: EventBusMessage) {
        let kind = msg.kind();
        let event_id = msg.event_id();

        match self.routes.read() {
            Ok(routes) => {
                for route in routes.iter() {
                    if !route.kinds.contains(&kind) {
                        continue;
                    }

                    match (route.deliver)(&msg) {
                        Outcome::Sent => {
                            tracing::debug!("[{event_id}] forwarded {kind} to {}", route.label)
                        }
                        Outcome::Skipped => {
                            tracing::trace!("[{event_id}] {} skipped {kind}", route.label)
                        }
                        Outcome::Dead => {
                            tracing::warn!("[{event_id}] {} is stopped", route.label)
                        }
                        Outcome::Failed(e) => {
                            tracing::warn!("[{event_id}] {} cast failed: {e}", route.label)
                        }
                    }
                }
            }
            Err(e) => tracing::error!("[{event_id}] event routes unreadable: {e}"),
        }

        if self.tx.send(msg).is_err() {
            tracing::trace!("[{event_id}] no stream subscribers for {kind} event");
        }
    }

    /// Subscribe a new receiver. Each subscriber sees every event published
    /// after it subscribed.
    pub fn subscribe(&self) -> broadcast::Receiver<EventBusMessage> {
        self.tx.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        // 1024 is generous for a home-automation event rate; lagging here would
        // mean ~1000 unhandled events backed up, which warrants the warning.
        Self::new(1024)
    }
}
