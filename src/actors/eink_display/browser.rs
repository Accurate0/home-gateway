use crate::eink::manager::source::ScreenshotBackend;
use crate::error::AppError;
use chromiumoxide::{
    Browser, BrowserConfig,
    cdp::browser_protocol::{
        emulation::{SetDeviceMetricsOverrideParams, SetLocaleOverrideParams},
        page::CaptureScreenshotFormat,
    },
    handler::viewport::Viewport,
    page::ScreenshotParams,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::task::JoinHandle;

pub struct Chromium {
    browser: Option<Browser>,
    #[allow(unused)]
    handle: Option<JoinHandle<()>>,
}

impl Chromium {
    pub async fn launch() -> Self {
        let config = BrowserConfig::builder()
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
            .build();

        let config = match config {
            Ok(config) => config,
            Err(e) => {
                tracing::warn!("chromium config invalid, screenshots disabled: {e}");
                return Self {
                    browser: None,
                    handle: None,
                };
            }
        };

        match Browser::launch(config).await {
            Ok((browser, mut handler)) => {
                let handle = tokio::spawn(async move {
                    while let Some(event) = handler.next().await {
                        if event.is_err() {
                            break;
                        }
                    }
                });

                Self {
                    browser: Some(browser),
                    handle: Some(handle),
                }
            }
            Err(e) => {
                tracing::warn!("chromium failed to launch, screenshots disabled: {e}");
                Self {
                    browser: None,
                    handle: None,
                }
            }
        }
    }

    pub fn is_available(&self) -> bool {
        self.browser.is_some()
    }

    pub async fn close(&mut self) -> Result<(), AppError> {
        if let Some(browser) = &mut self.browser {
            browser.close().await.map_err(anyhow::Error::from)?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl ScreenshotBackend for Chromium {
    async fn screenshot(
        &self,
        url: &str,
        dims: (u32, u32),
        settle: Duration,
    ) -> Result<Option<Vec<u8>>, AppError> {
        let Some(browser) = &self.browser else {
            tracing::warn!("skipping screenshot: chromium not available");
            return Ok(None);
        };

        let original_page = browser.new_page(url).await.map_err(anyhow::Error::from)?;
        tracing::info!("navigating to page");

        let (width, height) = dims;
        original_page
            .execute(
                SetDeviceMetricsOverrideParams::builder()
                    .width(width as i64)
                    .height(height as i64)
                    .device_scale_factor(1.0)
                    .mobile(false)
                    .build()
                    .map_err(|e| anyhow::anyhow!("{e}"))?,
            )
            .await
            .map_err(anyhow::Error::from)?;

        tracing::info!("setting locale and timezone");
        let page = original_page
            .emulate_timezone("Australia/Perth")
            .await
            .map_err(anyhow::Error::from)?;
        let page = page
            .emulate_locale(SetLocaleOverrideParams::builder().locale("en-AU").build())
            .await
            .map_err(anyhow::Error::from)?;
        page.reload().await.map_err(anyhow::Error::from)?;

        tokio::time::sleep(settle).await;

        let image = page
            .screenshot(
                ScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .full_page(false)
                    .build(),
            )
            .await
            .map_err(anyhow::Error::from)?;
        tracing::info!("screenshot taken");

        original_page.close().await.map_err(anyhow::Error::from)?;

        Ok(Some(image))
    }
}
