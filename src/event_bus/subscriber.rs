use std::sync::{Arc, RwLock, Weak};

use ractor::ActorRef;

use crate::actors::system::rpc;

use super::message::BusEvent;

pub trait EventSubscriber: Send + Sync + 'static {
    type Msg: ractor::Message;

    const KINDS: &'static [&'static str];

    fn to_actor_message(&self, event: &BusEvent) -> Option<Self::Msg>;
}

pub enum Recipient<M: ractor::Message> {
    Actor(ActorRef<M>),
    Factory(&'static str),
}

pub enum Outcome {
    Skipped,
    Sent,
    Dead,
    Failed(String),
}

pub struct Route {
    pub id: u64,
    pub label: &'static str,
    pub kinds: &'static [&'static str],
    pub deliver: Box<dyn Fn(&BusEvent) -> Outcome + Send + Sync>,
}

pub fn route<S: EventSubscriber>(
    id: u64,
    label: &'static str,
    recipient: Recipient<S::Msg>,
    subscriber: S,
) -> Route {
    Route {
        id,
        label,
        kinds: S::KINDS,
        deliver: Box::new(move |event| {
            let Some(message) = subscriber.to_actor_message(event) else {
                return Outcome::Skipped;
            };

            match &recipient {
                Recipient::Actor(actor) => {
                    if actor.get_status() == ractor::ActorStatus::Stopped {
                        return Outcome::Dead;
                    }

                    match actor.send_message(message) {
                        Ok(()) => Outcome::Sent,
                        Err(e) => Outcome::Failed(e.to_string()),
                    }
                }
                Recipient::Factory(name) => match rpc::cast_factory(name, message) {
                    Ok(()) => Outcome::Sent,
                    Err(e) => Outcome::Failed(e.to_string()),
                },
            }
        }),
    }
}

pub struct Subscription {
    id: u64,
    routes: Weak<RwLock<Vec<Route>>>,
}

impl Subscription {
    pub fn new(id: u64, routes: &Arc<RwLock<Vec<Route>>>) -> Self {
        Self {
            id,
            routes: Arc::downgrade(routes),
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        let Some(routes) = self.routes.upgrade() else {
            return;
        };

        if let Ok(mut routes) = routes.write() {
            routes.retain(|route| route.id != self.id);
        }
    }
}

impl Default for Subscription {
    /// A handle bound to no route, so tests can build actor state without a bus.
    fn default() -> Self {
        Self {
            id: 0,
            routes: Weak::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::{EventBus, EventBusMessage};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Counter {
        seen: Arc<AtomicUsize>,
    }

    impl EventSubscriber for Counter {
        type Msg = ();

        const KINDS: &'static [&'static str] = &["cron"];

        fn to_actor_message(&self, _event: &BusEvent) -> Option<()> {
            self.seen.fetch_add(1, Ordering::Relaxed);

            None
        }
    }

    fn cron_event() -> EventBusMessage {
        EventBusMessage::Cron {
            event_id: uuid::Uuid::new_v4(),
            name: "test".to_owned(),
        }
    }

    fn mode_event() -> EventBusMessage {
        EventBusMessage::Mode {
            event_id: uuid::Uuid::new_v4(),
            mode: crate::mode::Mode::Away,
            active: true,
        }
    }

    #[test]
    fn a_dropped_subscription_stops_delivery() {
        let bus = EventBus::default();
        let seen = Arc::new(AtomicUsize::new(0));

        let subscription = bus.register(
            "counter",
            Recipient::<()>::Factory("nobody"),
            Counter { seen: seen.clone() },
        );

        bus.publish(cron_event());
        assert_eq!(seen.load(Ordering::Relaxed), 1);

        drop(subscription);

        bus.publish(cron_event());
        assert_eq!(seen.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn an_unmatched_kind_never_reaches_the_mapper() {
        let bus = EventBus::default();
        let seen = Arc::new(AtomicUsize::new(0));

        let _subscription = bus.register(
            "counter",
            Recipient::<()>::Factory("nobody"),
            Counter { seen: seen.clone() },
        );

        bus.publish(mode_event());

        assert_eq!(seen.load(Ordering::Relaxed), 0);
    }
}
