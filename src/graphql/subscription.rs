use async_graphql::{Context, Result, SimpleObject, Subscription, Union};
use futures::{Stream, StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::auth::AuthContext;
use crate::auth::scope::{Action, Domain, Resource, Scope};
use crate::event_bus::{EventBus, EventBusMessage, EventFilter};

#[derive(SimpleObject)]
pub struct PresenceUpdate {
    pub event_id: Uuid,
    pub sensor: String,
    pub present: bool,
}

#[derive(SimpleObject)]
pub struct DoorUpdate {
    pub event_id: Uuid,
    pub device: String,
    pub open: bool,
}

#[derive(SimpleObject)]
pub struct SwitchUpdate {
    pub event_id: Uuid,
    pub device: String,
    pub action: String,
}

#[derive(SimpleObject)]
pub struct EnvironmentUpdate {
    pub event_id: Uuid,
    pub sensor: String,
    pub metric: String,
    pub value: f64,
}

#[derive(SimpleObject)]
pub struct CronUpdate {
    pub event_id: Uuid,
    pub name: String,
}

#[derive(Union)]
pub enum EventUpdate {
    Presence(PresenceUpdate),
    Door(DoorUpdate),
    Switch(SwitchUpdate),
    Environment(EnvironmentUpdate),
    Cron(CronUpdate),
}

impl From<EventBusMessage> for EventUpdate {
    fn from(msg: EventBusMessage) -> Self {
        match msg {
            EventBusMessage::Presence {
                event_id,
                sensor,
                present,
            } => EventUpdate::Presence(PresenceUpdate {
                event_id,
                sensor,
                present,
            }),
            EventBusMessage::Door {
                event_id,
                ieee_addr,
                open,
            } => EventUpdate::Door(DoorUpdate {
                event_id,
                device: ieee_addr.to_string(),
                open,
            }),
            EventBusMessage::SwitchAction {
                event_id,
                ieee_addr,
                action,
            } => EventUpdate::Switch(SwitchUpdate {
                event_id,
                device: ieee_addr.to_string(),
                action,
            }),
            EventBusMessage::Environment {
                event_id,
                sensor,
                reading,
            } => EventUpdate::Environment(EnvironmentUpdate {
                event_id,
                sensor,
                metric: format!("{:?}", reading.metric()),
                value: reading.value(),
            }),
            EventBusMessage::Cron { event_id, name } => {
                EventUpdate::Cron(CronUpdate { event_id, name })
            }
        }
    }
}

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
        Ok(event_stream(rx, filter))
    }
}

fn event_stream(
    rx: broadcast::Receiver<EventBusMessage>,
    filter: EventFilter,
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
        async move { keep.then(|| EventUpdate::from(msg)) }
    })
}
