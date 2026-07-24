use crate::s3::OptionalObjectResponse;
use crate::types::SharedActorState;
use chromiumoxide::{
    Browser, BrowserConfig,
    cdp::browser_protocol::{emulation::SetLocaleOverrideParams, page::CaptureScreenshotFormat},
    handler::viewport::Viewport,
    page::ScreenshotParams,
};
use futures::StreamExt;
use ractor::Actor;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;

pub mod types;

pub enum EInkDisplayMessage {
    TakeScreenshot {
        device_id: Option<String>,
    },
    BatteryReport {
        device_id: String,
        battery_voltage: f64,
    },
}

pub struct EInkDisplayActor {
    pub shared_actor_state: SharedActorState,
}

impl EInkDisplayActor {
    pub const NAME: &str = "eink_display";
    #[cfg(not(debug_assertions))]
    pub const INDEX_PATH: &str = "file:///tmp/index.html";
    pub const INDEX_FS_PATH: &str = "/tmp/index.html";
    #[cfg(debug_assertions)]
    pub const INDEX_PATH: &str =
        "file:///home/anurag/Projects/home-gateway/eink-display-web/dist/index.html";
}

pub struct EInkActorState {
    index_html_etag: Option<String>,
    // Optional so the gateway still starts where Chromium can't launch (e.g. a
    // dev box or sandbox with no usable /dev/shm or dbus). Screenshotting is
    // skipped in that case rather than crashing the whole process.
    browser: Option<Browser>,
    #[allow(unused)]
    browser_handle: Option<JoinHandle<()>>,
}

impl EInkDisplayActor {
    async fn save_index_if_new(
        &self,
        state: &mut EInkActorState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let etag = state.index_html_etag.as_deref();
        let optional_index = self
            .shared_actor_state
            .s3
            .get_object_optional("index.html", etag)
            .await?;

        match optional_index {
            OptionalObjectResponse::ExistingObjectIsValid => {
                tracing::info!("304 from server, not replacing file");
            }
            OptionalObjectResponse::ObjectUpdated(object_response) => {
                tracing::info!("index fetched: etag {:?}", object_response.etag);
                let mut index_file = File::create(Self::INDEX_FS_PATH).await?;
                index_file.write_all(&object_response.payload).await?;
                state.index_html_etag = Some(object_response.etag);
            }
        };

        Ok(())
    }

    async fn render_web(
        &self,
        state: &mut EInkActorState,
        settle: Duration,
    ) -> Result<Option<Vec<u8>>, ractor::ActorProcessingErr> {
        if !cfg!(debug_assertions) {
            self.save_index_if_new(state).await?;
        }

        let Some(browser) = &state.browser else {
            tracing::warn!("skipping screenshot: chromium not available");
            return Ok(None);
        };
        let original_page = browser.new_page(Self::INDEX_PATH).await?;
        tracing::info!("navigating to page");

        tracing::info!("setting locale and timezone");
        let page = original_page.emulate_timezone("Australia/Perth").await?;
        let page = page
            .emulate_locale(SetLocaleOverrideParams::builder().locale("en-AU").build())
            .await?;
        page.reload().await?;

        tokio::time::sleep(settle).await;

        tracing::info!("screenshot taken");
        let image = page
            .screenshot(
                ScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .full_page(false)
                    .build(),
            )
            .await?;

        original_page.close().await?;

        Ok(Some(image))
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
        let default_refresh = Duration::from_secs(3600);

        for (device_id, display) in self.shared_actor_state.devices.eink_displays() {
            let refresh = display
                .refresh
                .and_then(|r| r.to_std().ok())
                .unwrap_or(default_refresh);
            let device_id = device_id.clone();
            let _join_handle =
                myself.send_interval(refresh, move || EInkDisplayMessage::TakeScreenshot {
                    device_id: Some(device_id.clone()),
                });
        }

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
                emulating_mobile: true,
                is_landscape: true,
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

        if browser.is_some() {
            myself.send_message(EInkDisplayMessage::TakeScreenshot { device_id: None })?;
        }

        Ok(EInkActorState {
            browser,
            index_html_etag: None,
            browser_handle,
        })
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
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            EInkDisplayMessage::TakeScreenshot { device_id } => {
                let all = self.shared_actor_state.devices.eink_displays();
                let targets: Vec<(String, String)> = match &device_id {
                    Some(id) => match all.get(id) {
                        Some(display) => vec![(id.clone(), display.name.clone())],
                        None => {
                            tracing::warn!("render requested for unknown eink display '{id}'");
                            return Ok(());
                        }
                    },
                    None => all
                        .iter()
                        .map(|(id, display)| (id.clone(), display.name.clone()))
                        .collect(),
                };
                if targets.is_empty() {
                    tracing::debug!("no registered eink displays, skipping render");
                    return Ok(());
                }

                let settle = device_id
                    .as_ref()
                    .and_then(|id| all.get(id))
                    .and_then(|display| display.settle)
                    .and_then(|s| s.to_std().ok())
                    .unwrap_or(Duration::from_secs(10));

                let image = match self.render_web(state, settle).await? {
                    Some(image) => image,
                    None => return Ok(()),
                };

                for (device_id, name) in targets {
                    let key = format!("eink-display/image-{device_id}.png");
                    self.shared_actor_state
                        .s3
                        .put_object(&key, &image, None)
                        .await?;

                    sqlx::query!(
                        "INSERT INTO eink_display (device_id, name, image_key, updated_at) VALUES ($1, $2, $3, now()) \
                         ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, image_key = EXCLUDED.image_key, updated_at = EXCLUDED.updated_at",
                        device_id,
                        name,
                        key,
                    )
                    .execute(&self.shared_actor_state.db)
                    .await?;

                    tracing::info!("eink display image uploaded for {device_id} -> {key}");
                }
            }
            EInkDisplayMessage::BatteryReport {
                device_id,
                battery_voltage,
            } => {
                let Some(display) = self.shared_actor_state.devices.eink_display(&device_id) else {
                    tracing::warn!(
                        "battery report from unregistered eink display '{device_id}', dropping"
                    );
                    return Ok(());
                };
                let name = display.name.clone();
                let event_id = uuid::Uuid::new_v4();
                let kind = "eink_display_firmware";

                sqlx::query!(
                    "INSERT INTO device_battery (event_id, device_id, kind, battery_voltage) VALUES ($1, $2, $3, $4)",
                    event_id,
                    device_id,
                    kind,
                    battery_voltage,
                )
                .execute(&self.shared_actor_state.db)
                .await?;

                sqlx::query!(
                    "INSERT INTO eink_display (device_id, name, battery_voltage, updated_at) VALUES ($1, $2, $3, now()) \
                     ON CONFLICT (device_id) DO UPDATE SET name = EXCLUDED.name, battery_voltage = EXCLUDED.battery_voltage, updated_at = EXCLUDED.updated_at",
                    device_id,
                    name,
                    battery_voltage,
                )
                .execute(&self.shared_actor_state.db)
                .await?;

                crate::metrics::record_device_battery_voltage(
                    device_id.clone(),
                    kind.to_owned(),
                    battery_voltage,
                );

                self.shared_actor_state.event_bus.publish(
                    crate::event_bus::EventBusMessage::DeviceBattery {
                        event_id,
                        device_id,
                        kind: kind.to_owned(),
                        name,
                        battery_voltage,
                    },
                );
            }
        }

        Ok(())
    }
}
