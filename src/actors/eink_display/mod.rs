use crate::battery::{BatteryChemistry, voltage_to_percentage};
use crate::eink::manager::EinkDisplayManager;
use crate::eink::manager::source::Prepared;
use crate::error::AppError;
use crate::state::AppState;
use browser::Chromium;
use ractor::{Actor, RpcReplyPort};
use std::collections::HashMap;

mod browser;
mod schedule;

pub enum EInkDisplayMessage {
    TakeScreenshot {
        device_id: Option<String>,
    },
    PrepareRender {
        device_id: String,
        reply: RpcReplyPort<()>,
    },
    BatteryReport {
        device_id: String,
        battery_voltage: f64,
        is_charging: Option<bool>,
        battery_chemistry: Option<BatteryChemistry>,
        battery_kind: Option<String>,
    },
    ConfigRequest {
        device_id: String,
    },
    ScheduleNextRender {
        device_id: String,
        wake_in_secs: u32,
    },
}

pub struct EInkDisplayActor {
    pub shared_actor_state: AppState,
}

pub struct EInkActorState {
    browser: Chromium,
    scheduled_renders: HashMap<String, tokio::task::AbortHandle>,
}

impl EInkDisplayActor {
    pub const NAME: &str = "eink_display";

    fn manager(&self) -> &EinkDisplayManager {
        self.shared_actor_state
            .handles
            .expect::<EinkDisplayManager>()
    }

    async fn take_screenshot(
        &self,
        state: &EInkActorState,
        device_id: Option<String>,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let manager = self.manager();

        let target_ids = match device_id {
            Some(id) => match manager.resolve(&id).await.is_some() {
                true => vec![id],
                false => {
                    tracing::warn!("render requested for unknown eink display '{id}'");
                    return Ok(());
                }
            },
            None => manager.device_ids(),
        };

        if target_ids.is_empty() {
            tracing::debug!("no registered eink displays, skipping render");
            return Ok(());
        }

        for device_id in target_ids {
            let Some(display) = manager.resolve(&device_id).await else {
                continue;
            };

            manager
                .refresh_source(&display, &state.browser)
                .await
                .map_err(AppError::message)?;
        }

        Ok(())
    }

    async fn prepare_render(
        &self,
        myself: &ractor::ActorRef<EInkDisplayMessage>,
        state: &EInkActorState,
        device_id: &str,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let Some(display) = self.manager().resolve(device_id).await else {
            tracing::warn!("render requested for unregistered eink display '{device_id}'");
            return Ok(());
        };

        let prepared = self
            .manager()
            .prepare_source(&display, &state.browser)
            .await
            .map_err(AppError::message)?;

        if prepared == Prepared::NeedsRefresh {
            myself.send_message(EInkDisplayMessage::TakeScreenshot {
                device_id: Some(device_id.to_owned()),
            })?;
        }

        Ok(())
    }
}

impl Actor for EInkDisplayActor {
    type Msg = EInkDisplayMessage;
    type State = EInkActorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let mut state = EInkActorState {
            browser: Chromium::launch().await,
            scheduled_renders: HashMap::new(),
        };

        if !state.browser.is_available() {
            return Ok(state);
        }

        for device_id in self.manager().device_ids() {
            let Some(display) = self.manager().resolve(&device_id).await else {
                continue;
            };

            if display.mode != crate::settings::EinkMode::Dashboard {
                continue;
            }

            let next_wake_at = self
                .manager()
                .stored_next_wake(&device_id)
                .await
                .map_err(AppError::message)?;

            self.schedule_render(&myself, &mut state, &display, next_wake_at)?;
        }

        Ok(state)
    }

    async fn post_stop(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        state.browser.close().await.map_err(AppError::message)?;

        Ok(())
    }

    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            EInkDisplayMessage::TakeScreenshot { device_id } => {
                self.take_screenshot(state, device_id).await?;
            }
            EInkDisplayMessage::PrepareRender { device_id, reply } => {
                let result = self.prepare_render(&myself, state, &device_id).await;

                if reply.send(()).is_err() {
                    tracing::warn!("prepare render reply dropped for {device_id}");
                }

                result?;
            }
            EInkDisplayMessage::BatteryReport {
                device_id,
                battery_voltage,
                is_charging,
                battery_chemistry,
                battery_kind,
            } => {
                let Some(display) = self.shared_actor_state.devices.eink_display(&device_id) else {
                    tracing::warn!(
                        "battery report from unregistered eink display '{device_id}', dropping"
                    );
                    return Ok(());
                };
                let name = display.name.clone();
                let kind = battery_kind.unwrap_or_else(|| "eink_display_firmware".to_owned());
                let chemistry = battery_chemistry.unwrap_or(BatteryChemistry::Unknown);

                self.manager()
                    .store_battery(&device_id, &name, battery_voltage, is_charging)
                    .await
                    .map_err(AppError::message)?;

                crate::actors::system::battery::BatteryActor::report(
                    device_id,
                    name,
                    kind,
                    Some(battery_voltage),
                    voltage_to_percentage(chemistry, battery_voltage),
                    Some(chemistry),
                );
            }
            EInkDisplayMessage::ConfigRequest { device_id } => {
                let Some(display) = self.shared_actor_state.devices.eink_display(&device_id) else {
                    tracing::warn!(
                        "config request from unregistered eink display '{device_id}', dropping"
                    );
                    return Ok(());
                };

                self.manager()
                    .store_seen(&device_id, &display.name)
                    .await
                    .map_err(AppError::message)?;
            }
            EInkDisplayMessage::ScheduleNextRender {
                device_id,
                wake_in_secs,
            } => {
                let Some(display) = self.manager().resolve(&device_id).await else {
                    tracing::warn!(
                        "next wake reported for unregistered eink display '{device_id}', dropping"
                    );
                    return Ok(());
                };

                let next_wake_at =
                    chrono::Utc::now() + chrono::TimeDelta::seconds(wake_in_secs.into());

                self.manager()
                    .store_next_wake(&device_id, &display.name, next_wake_at)
                    .await
                    .map_err(AppError::message)?;

                if display.mode != crate::settings::EinkMode::Dashboard {
                    Self::cancel_render(state, &device_id);
                    return Ok(());
                }

                self.schedule_render(&myself, state, &display, Some(next_wake_at))?;
            }
        }

        Ok(())
    }
}
