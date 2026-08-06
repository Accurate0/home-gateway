use crate::battery::{BatteryChemistry, voltage_to_percentage};
use crate::reddit::Reddit;
use crate::settings::EinkMode;
use crate::state::SharedActorState;
use chromiumoxide::{Browser, BrowserConfig, handler::viewport::Viewport};
use futures::StreamExt;
use ractor::{Actor, RpcReplyPort};
use render::content_hash;
use std::collections::HashMap;
use std::time::Duration;
use tokio::task::JoinHandle;

const CACHE_PREFIX: &str = "eink-display/cache/";
const FLAG_OVERRIDE_SETTLE: Duration = Duration::from_secs(10);

mod render;
mod schedule;
mod store;
pub mod types;

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
        wake_in_mins: u32,
    },
}

pub struct EInkDisplayActor {
    pub shared_actor_state: SharedActorState,
    pub reddit: Reddit,
}

pub struct EInkActorState {
    index_html_etag: Option<String>,
    // Optional so the gateway still starts where Chromium can't launch (e.g. a
    // dev box or sandbox with no usable /dev/shm or dbus). Screenshotting is
    // skipped in that case rather than crashing the whole process.
    browser: Option<Browser>,
    #[allow(unused)]
    browser_handle: Option<JoinHandle<()>>,
    scheduled_renders: HashMap<String, tokio::task::AbortHandle>,
}

impl EInkDisplayActor {
    pub const NAME: &str = "eink_display";
    #[cfg(not(debug_assertions))]
    pub const INDEX_PATH: &str = "file:///tmp/index.html";
    pub const INDEX_FS_PATH: &str = "/tmp/index.html";
    #[cfg(debug_assertions)]
    pub const INDEX_PATH: &str =
        "file:///home/anurag/Projects/home-gateway/eink-display-web/dist/index.html";
    async fn prepare_render(
        &self,
        myself: &ractor::ActorRef<EInkDisplayMessage>,
        device_id: &str,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let Some(display) = self.shared_actor_state.devices.eink_display(device_id) else {
            tracing::warn!("render requested for unregistered eink display '{device_id}'");
            return Ok(());
        };
        let display = display.clone();

        let flag = crate::epd::epd_flag_config(
            &self.shared_actor_state.feature_flag_client,
            &self.shared_actor_state.devices,
            device_id,
        )
        .await;

        let global = &self.shared_actor_state.settings.eink_display;

        match flag.resolve_mode(&display) {
            EinkMode::Album => {
                let album = flag.resolve_album(global, &display);
                if let Some((image_key, hash)) = self.render_album(&display, &album).await? {
                    self.store_render(device_id, &display.name, &image_key, &hash)
                        .await?;
                }
            }
            EinkMode::Reddit => {
                if let Some(feed) = flag.resolve_reddit_feed(&display)
                    && let Some((image_key, hash)) = self.render_reddit(&display, &feed).await?
                {
                    self.store_render(device_id, &display.name, &image_key, &hash)
                        .await?;
                }
            }
            EinkMode::Dashboard => {
                let view_name = flag
                    .resolve_view(global, &display)
                    .map(|view| view.name.clone())
                    .unwrap_or_else(|| "default".to_owned());
                let image_key = Self::dashboard_key(&view_name, display.orientation_str());

                let adopted = self
                    .adopt_last_dashboard(device_id, &display, &image_key)
                    .await?;

                if !adopted {
                    myself.send_message(EInkDisplayMessage::TakeScreenshot {
                        device_id: Some(device_id.to_owned()),
                    })?;
                }
            }
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
        let launch = BrowserConfig::builder()
            .new_headless_mode()
            .arg("--disable-crash-reporter")
            .arg("--no-crashpad")
            .arg("--no-sandbox")
            // container has a small /dev/shm; without this Chromium crashes on startup (SIGTRAP)
            .arg("--disable-dev-shm-usage")
            .arg("--disable-gpu")
            .env("XDG_CONFIG_HOME", "/tmp/chromium")
            .env("XDG_CACHE_HOME", "/tmp/chromium")
            .viewport(Some(Viewport {
                width: 1600,
                height: 1200,
                device_scale_factor: None,
                emulating_mobile: false,
                is_landscape: false,
                has_touch: false,
            }))
            .build()
            .map_err(ractor::ActorProcessingErr::from);

        let (browser, browser_handle) = match launch {
            Ok(config) => match Browser::launch(config).await {
                Ok((browser, mut handler)) => {
                    let handle = tokio::spawn(async move {
                        while let Some(h) = handler.next().await {
                            if h.is_err() {
                                break;
                            }
                        }
                    });
                    (Some(browser), Some(handle))
                }
                Err(e) => {
                    tracing::warn!("chromium failed to launch, screenshots disabled: {e}");
                    (None, None)
                }
            },
            Err(e) => {
                tracing::warn!("chromium config invalid, screenshots disabled: {e}");
                (None, None)
            }
        };

        let mut state = EInkActorState {
            browser,
            index_html_etag: None,
            browser_handle,
            scheduled_renders: HashMap::new(),
        };

        if state.browser.is_some() {
            let device_ids: Vec<String> = self
                .shared_actor_state
                .devices
                .eink_displays()
                .keys()
                .cloned()
                .collect();

            for device_id in device_ids {
                let Some(display) = self.shared_actor_state.devices.eink_display(&device_id) else {
                    continue;
                };
                let display = display.clone();

                let flag = crate::epd::epd_flag_config(
                    &self.shared_actor_state.feature_flag_client,
                    &self.shared_actor_state.devices,
                    &device_id,
                )
                .await;

                if flag.resolve_mode(&display) != EinkMode::Dashboard {
                    continue;
                }

                let next_wake_at = self.get_stored_next_wake(&device_id).await?;

                self.schedule_render(&myself, &mut state, &device_id, &display, next_wake_at)?;
            }
        }

        Ok(state)
    }

    async fn post_stop(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        if let Some(browser) = &mut state.browser {
            browser.close().await?;
        }
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
                let all = self.shared_actor_state.devices.eink_displays();
                let target_ids: Vec<String> = match &device_id {
                    Some(id) => {
                        if all.contains_key(id) {
                            vec![id.clone()]
                        } else {
                            tracing::warn!("render requested for unknown eink display '{id}'");
                            return Ok(());
                        }
                    }
                    None => all.keys().cloned().collect(),
                };
                if target_ids.is_empty() {
                    tracing::debug!("no registered eink displays, skipping render");
                    return Ok(());
                }

                for device_id in target_ids {
                    let display = match self.shared_actor_state.devices.eink_display(&device_id) {
                        Some(display) => display.clone(),
                        None => continue,
                    };

                    let flag = crate::epd::epd_flag_config(
                        &self.shared_actor_state.feature_flag_client,
                        &self.shared_actor_state.devices,
                        &device_id,
                    )
                    .await;

                    let global = &self.shared_actor_state.settings.eink_display;
                    let (image_key, image_content_hash) = match flag.resolve_mode(&display) {
                        EinkMode::Dashboard => {
                            let settle = display
                                .mode
                                .settle()
                                .and_then(|s| s.to_std().ok())
                                .unwrap_or(FLAG_OVERRIDE_SETTLE);
                            let view = flag.resolve_view(global, &display);
                            let view_name = view
                                .map(|v| v.name.clone())
                                .unwrap_or_else(|| "default".to_owned());
                            let query = view.and_then(|v| v.query.clone());
                            let image = match self
                                .render_web(state, &display, settle, query.as_deref())
                                .await?
                            {
                                Some(image) => image,
                                None => continue,
                            };
                            let key = Self::dashboard_key(&view_name, display.orientation_str());
                            self.shared_actor_state
                                .s3
                                .put_object(&key, &image, Some("image/png"))
                                .await?;

                            let hash = content_hash(&image);

                            (key, hash)
                        }
                        EinkMode::Album => match self
                            .render_album(&display, &flag.resolve_album(global, &display))
                            .await?
                        {
                            Some(rendered) => rendered,
                            None => continue,
                        },
                        EinkMode::Reddit => {
                            let Some(feed) = flag.resolve_reddit_feed(&display) else {
                                continue;
                            };
                            match self.render_reddit(&display, &feed).await? {
                                Some(rendered) => rendered,
                                None => continue,
                            }
                        }
                    };

                    self.store_render(&device_id, &display.name, &image_key, &image_content_hash)
                        .await?;
                }
            }
            EInkDisplayMessage::PrepareRender { device_id, reply } => {
                let result = self.prepare_render(&myself, &device_id).await;

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

                sqlx::query!(
                    "INSERT INTO eink_display (device_id, name, battery_voltage, is_charging, updated_at) VALUES ($1, $2, $3, $4, now()) \
                     ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, battery_voltage = EXCLUDED.battery_voltage, is_charging = EXCLUDED.is_charging, updated_at = EXCLUDED.updated_at",
                    device_id,
                    name,
                    battery_voltage,
                    is_charging,
                )
                .execute(&self.shared_actor_state.db)
                .await?;

                crate::actors::battery::BatteryActor::report(
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

                sqlx::query!(
                    "INSERT INTO eink_display (device_id, name, updated_at) VALUES ($1, $2, now()) \
                     ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, updated_at = EXCLUDED.updated_at",
                    device_id,
                    display.name,
                )
                .execute(&self.shared_actor_state.db)
                .await?;
            }
            EInkDisplayMessage::ScheduleNextRender {
                device_id,
                wake_in_mins,
            } => {
                let Some(display) = self.shared_actor_state.devices.eink_display(&device_id) else {
                    tracing::warn!(
                        "next wake reported for unregistered eink display '{device_id}', dropping"
                    );
                    return Ok(());
                };
                let display = display.clone();

                let next_wake_at =
                    chrono::Utc::now() + chrono::TimeDelta::minutes(wake_in_mins.into());

                self.store_next_wake(&device_id, &display.name, next_wake_at)
                    .await?;

                let flag = crate::epd::epd_flag_config(
                    &self.shared_actor_state.feature_flag_client,
                    &self.shared_actor_state.devices,
                    &device_id,
                )
                .await;

                if flag.resolve_mode(&display) != EinkMode::Dashboard {
                    Self::cancel_render(state, &device_id);
                    return Ok(());
                }

                self.schedule_render(&myself, state, &device_id, &display, Some(next_wake_at))?;
            }
        }

        Ok(())
    }
}
