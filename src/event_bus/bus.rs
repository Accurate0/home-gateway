use tokio::sync::broadcast;

use super::message::EventBusMessage;

/// Clonable handle to the in-memory event bus. Cheap to clone (shares one
/// broadcast sender). Stored on `AppState` so any actor can publish.
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<EventBusMessage>,
}

impl EventBus {
    /// `capacity` bounds the per-subscriber backlog; slow subscribers that fall
    /// behind observe a `Lagged` error rather than blocking producers.
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publish an event. Failure means there are currently no subscribers, which
    /// is not an error for a fire-and-forget bus.
    pub fn publish(&self, msg: EventBusMessage) {
        let kind = msg.kind();
        let event_id = msg.event_id();
        if self.tx.send(msg).is_err() {
            tracing::debug!("[{event_id}] no subscribers for {kind} event");
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
