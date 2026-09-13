use crate::eink::manager::source::ScreenshotBackend;
use crate::error::AppError;
use chromiumoxide::{
    Browser, BrowserConfig, Page,
    cdp::browser_protocol::{
        emulation::{SetDeviceMetricsOverrideParams, SetLocaleOverrideParams},
        page::CaptureScreenshotFormat,
    },
    handler::viewport::Viewport,
    page::ScreenshotParams,
};
use futures::StreamExt;
use std::time::Duration;

pub struct Chromium;

impl Chromium {
    fn config() -> Result<BrowserConfig, String> {
        BrowserConfig::builder()
            .new_headless_mode()
            .arg("--disable-crash-reporter")
            .arg("--no-crashpad")
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-gpu")
            .arg("--disable-extensions")
            .arg("--disable-background-networking")
            .arg("--renderer-process-limit=1")
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
    }

    async fn capture(
        page: &Page,
        dims: (u32, u32),
        settle: Duration,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let (width, height) = dims;

        page.execute(
            SetDeviceMetricsOverrideParams::builder()
                .width(width as i64)
                .height(height as i64)
                .device_scale_factor(1.0)
                .mobile(false)
                .build()
                .map_err(|e| anyhow::anyhow!("{e}"))?,
        )
        .await?;

        tracing::info!("setting locale and timezone");
        let page = page.emulate_timezone("Australia/Perth").await?;
        let page = page
            .emulate_locale(SetLocaleOverrideParams::builder().locale("en-AU").build())
            .await?;
        page.reload().await?;

        tokio::time::sleep(settle).await;

        let image = page
            .screenshot(
                ScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .full_page(false)
                    .build(),
            )
            .await?;
        tracing::info!("screenshot taken");

        Ok(image)
    }

    async fn shutdown(mut browser: Browser) {
        if let Err(e) = browser.close().await {
            tracing::warn!("chromium close failed, killing: {e}");

            if let Some(Err(e)) = browser.kill().await {
                tracing::warn!("chromium kill failed: {e}");
            }
        }

        if let Err(e) = browser.wait().await {
            tracing::warn!("chromium wait failed: {e}");
        }
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
        let config = match Self::config() {
            Ok(config) => config,
            Err(e) => {
                tracing::warn!("chromium config invalid, skipping screenshot: {e}");
                return Ok(None);
            }
        };

        let (browser, mut handler) = match Browser::launch(config).await {
            Ok(launched) => launched,
            Err(e) => {
                tracing::warn!("chromium failed to launch, skipping screenshot: {e}");
                return Ok(None);
            }
        };

        let handle = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        let result = match browser.new_page(url).await {
            Ok(page) => {
                tracing::info!("navigating to page");
                let result = Self::capture(&page, dims, settle).await;

                if let Err(e) = page.close().await {
                    tracing::warn!("closing screenshot page failed: {e}");
                }

                result
            }
            Err(e) => Err(e.into()),
        };

        Self::shutdown(browser).await;
        handle.abort();

        Ok(Some(result?))
    }
}
