use std::time::Duration;

use chrono::Utc;
use ractor::Actor;
use tracing::Level;

use crate::{integrations::notify::notify, settings::NotifySource, state::AppState};

pub enum WatchdogMessage {
    Check,
}

pub struct WatchdogActor {
    pub shared_actor_state: AppState,
}

impl WatchdogActor {
    pub const NAME: &str = "watchdog";

    async fn check(&self) -> Result<(), anyhow::Error> {
        let watchdog = &self.shared_actor_state.settings.watchdog;
        let now = Utc::now();
        let realert_before = now - watchdog.realert_after;

        let seen = self.shared_actor_state.repos.watchdog().last_seen().await?;

        for (device_key, device) in self.shared_actor_state.devices.watchdog_devices() {
            let Some(state) = seen.get(device_key) else {
                continue;
            };

            let timeout = device.timeout.unwrap_or(watchdog.timeout);
            let targets: &[NotifySource] = if device.notify.is_empty() {
                std::slice::from_ref(&NotifySource::AndroidApp)
            } else {
                &device.notify
            };
            let is_stale = state.last_seen < now - timeout;

            if is_stale {
                let should_alert = state
                    .alerted_at
                    .is_none_or(|alerted_at| alerted_at < realert_before);
                if should_alert {
                    notify(
                        targets,
                        format!("Sensor offline: {device_key} has stopped reporting"),
                    );
                    self.shared_actor_state
                        .repos
                        .watchdog()
                        .mark_alerted(device_key, now)
                        .await?;
                }
            } else if state.alerted_at.is_some() {
                notify(
                    targets,
                    format!("Sensor back online: {device_key} is reporting again"),
                );
                self.shared_actor_state
                    .repos
                    .watchdog()
                    .clear_alerted(device_key)
                    .await?;
            }
        }

        Ok(())
    }
}

impl Actor for WatchdogActor {
    type Msg = WatchdogMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let watchdog = &self.shared_actor_state.settings.watchdog;
        if !watchdog.enabled {
            tracing::info!("sensor offline watchdog is disabled, not arming checks");
            return Ok(());
        }

        let interval = watchdog
            .check_interval
            .to_std()
            .unwrap_or(Duration::from_secs(300));
        let _join_handle = myself.send_interval(interval, || WatchdogMessage::Check);

        Ok(())
    }

    #[tracing::instrument(name = "watchdog-actor", skip(self, _myself, message, _state), level = Level::TRACE)]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            WatchdogMessage::Check => {
                if let Err(e) = self.check().await {
                    tracing::error!("watchdog check failed: {e}");
                }
            }
        }

        Ok(())
    }
}
