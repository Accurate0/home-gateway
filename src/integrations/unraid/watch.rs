use std::time::Instant;

use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::Unraid;
use super::types::{
    DiskSerials, DiskTemperature, UnraidEvent, UnraidNotification, UnraidPush, UnraidSnapshot,
};
use super::websocket::stream;
use crate::event_bus::{EventBus, EventBusMessage};
use crate::metrics::record_integration_poll;
use crate::repo::UnraidRepo;
use crate::settings::UnraidSettings;
use crate::telemetry::context::record_current_error;

struct UnraidState {
    previous: Option<UnraidSnapshot>,
    serials: DiskSerials,
}

struct UnraidWatcher {
    unraid: Unraid,
    repo: UnraidRepo,
    event_bus: EventBus,
    notification_backfill: i32,
}

pub async fn process_events(
    unraid: Unraid,
    settings: UnraidSettings,
    repo: UnraidRepo,
    event_bus: EventBus,
    cancellation_token: CancellationToken,
) {
    let previous = match repo.latest().await {
        Ok(stored) => stored.map(|stored| stored.snapshot),
        Err(e) => {
            tracing::error!("error loading the last unraid snapshot: {e}");
            None
        }
    };

    let mut state = UnraidState {
        previous,
        serials: DiskSerials::default(),
    };

    let watcher = UnraidWatcher {
        unraid,
        repo,
        event_bus,
        notification_backfill: i32::from(settings.notification_backfill),
    };

    let (pushes, mut received) = mpsc::unbounded_channel();
    let stream = stream(&watcher.unraid, &settings.websocket, pushes);
    tokio::pin!(stream);

    let mut refresh = tokio::time::interval(settings.refresh());
    refresh.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = refresh.tick() => watcher.poll(&mut state).await,
            Some(push) = received.recv() => match push {
                UnraidPush::Subscribed => watcher.backfill().await,
                UnraidPush::Temperatures(readings) => watcher.temperatures(&mut state, readings).await,
                UnraidPush::Notification(notification) => watcher.notification(notification).await,
            },
            _ = &mut stream => {
                tracing::error!("the unraid websocket stream stopped");
                return;
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("unraid watcher cancellation requested");
                return;
            }
        }
    }
}

impl UnraidWatcher {
    #[tracing::instrument(
        parent = None,
        name = "unraid.poll",
        skip(self, state),
        fields(
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        )
    )]
    async fn poll(&self, state: &mut UnraidState) {
        let started = Instant::now();

        match self.refresh(state).await {
            Ok(()) => {
                record_integration_poll("unraid", "success", started.elapsed());
            }
            Err(e) => {
                tracing::error!("error polling unraid: {e}");
                record_current_error(&e.to_string());
                record_integration_poll("unraid", "error", started.elapsed());
            }
        }
    }

    async fn refresh(&self, state: &mut UnraidState) -> Result<(), anyhow::Error> {
        let poll = self.unraid.snapshot().await?;

        state.serials = poll.serials;

        self.apply(state, poll.snapshot).await
    }

    async fn temperatures(&self, state: &mut UnraidState, readings: Vec<DiskTemperature>) {
        let Some(previous) = &state.previous else {
            tracing::debug!(
                "no unraid snapshot yet, dropping {} temperatures",
                readings.len()
            );
            return;
        };

        let snapshot = previous.with_temperatures(&state.serials, &readings);

        if snapshot == *previous {
            tracing::trace!("unraid temperatures unchanged");
            return;
        }

        self.record_temperatures(state, snapshot).await;
    }

    #[tracing::instrument(
        parent = None,
        name = "unraid.temperatures",
        skip(self, state, snapshot),
        fields(
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        )
    )]
    async fn record_temperatures(&self, state: &mut UnraidState, snapshot: UnraidSnapshot) {
        if let Err(e) = self.apply(state, snapshot).await {
            tracing::error!("error recording unraid temperatures: {e}");
            record_current_error(&e.to_string());
        }
    }

    #[tracing::instrument(
        parent = None,
        name = "unraid.backfill",
        skip(self),
        fields(
            otel.status_code = tracing::field::Empty,
            otel.status_message = tracing::field::Empty,
        )
    )]
    async fn backfill(&self) {
        if let Err(e) = self.backfill_unread().await {
            tracing::error!("error backfilling unraid notifications: {e}");
            record_current_error(&e.to_string());
        }
    }

    async fn backfill_unread(&self) -> Result<(), anyhow::Error> {
        let unread = self
            .unraid
            .unread_notifications(self.notification_backfill)
            .await?;

        if !self.repo.has_notifications().await? {
            for notification in &unread {
                self.repo.record_notification(notification, false).await?;
            }

            tracing::info!(
                "seeded {} unread unraid notifications without forwarding them",
                unread.len()
            );

            return Ok(());
        }

        let mut forwarded = 0;

        for notification in unread.into_iter().rev() {
            if self.forward(notification).await? {
                forwarded += 1;
            }
        }

        tracing::info!("backfilled {forwarded} unraid notifications");

        Ok(())
    }

    async fn notification(&self, notification: UnraidNotification) {
        if let Err(e) = self.forward(notification).await {
            tracing::error!("error recording an unraid notification: {e}");
        }
    }

    async fn forward(&self, notification: UnraidNotification) -> Result<bool, anyhow::Error> {
        if !self.repo.record_notification(&notification, true).await? {
            tracing::debug!("unraid notification {} already handled", notification.id);
            return Ok(false);
        }

        tracing::info!(
            "unraid raised a {} notification {}: {}",
            notification.importance,
            notification.id,
            notification.subject
        );

        self.publish(UnraidEvent::notification(notification));

        Ok(true)
    }

    fn publish(&self, event: UnraidEvent) {
        self.event_bus.publish(EventBusMessage::Unraid {
            event_id: Uuid::new_v4(),
            subject: event.subject,
            kind: event.kind,
            update: event.update,
        });
    }

    async fn apply(
        &self,
        state: &mut UnraidState,
        snapshot: UnraidSnapshot,
    ) -> Result<(), anyhow::Error> {
        self.repo.record(&snapshot).await?;

        let events = UnraidEvent::between(state.previous.as_ref(), &snapshot);

        tracing::info!(
            "recorded unraid array {} with {} disks, publishing {} events",
            snapshot.array.state,
            snapshot.disks.len(),
            events.len()
        );

        for event in events {
            self.publish(event);
        }

        state.previous = Some(snapshot);

        Ok(())
    }
}
