use crate::types::SharedActorState;
use chromiumoxide::{
    Browser, BrowserConfig,
    browser::HeadlessMode,
    cdp::browser_protocol::{emulation::SetLocaleOverrideParams, page::CaptureScreenshotFormat},
    handler::viewport::Viewport,
    page::ScreenshotParams,
};
use futures::StreamExt;
use ractor::Actor;
use std::time::Duration;
use tokio::{fs::File, io::AsyncWriteExt};

pub mod types;

pub enum EInkDisplayMessage {
    TakeScreenshot,
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

impl Actor for EInkDisplayActor {
    type Msg = EInkDisplayMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        // send initial before schedule
        let _join_handle = myself.send_after(Duration::from_secs(1), || {
            EInkDisplayMessage::TakeScreenshot
        });

        let _join_handle = myself.send_interval(Duration::from_secs(300), || {
            EInkDisplayMessage::TakeScreenshot
        });

        Ok(())
    }

    #[tracing::instrument(name = "eink-display-actor", skip(self, _myself, message, _state))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            EInkDisplayMessage::TakeScreenshot => {
                if !cfg!(debug_assertions) {
                    let index_html_file = self
                        .shared_actor_state
                        .object_registry
                        .get_object::<Vec<u8>>("home-gateway", "index.html")
                        .await?;

                    tracing::info!("index fetched: {:?}", index_html_file.metadata);
                    let mut index_file = File::create(Self::INDEX_FS_PATH).await?;
                    index_file.write_all(&index_html_file.payload).await?;
                }

                tracing::info!("starting browser");
                let (mut browser, mut handler) = Browser::launch(
                    BrowserConfig::builder()
                        .headless_mode(HeadlessMode::New)
                        .arg("--disable-crash-reporter")
                        .arg("--no-crashpad")
                        .arg("--no-sandbox")
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
                        .build()?,
                )
                .await?;
                tracing::info!("browser started");

                let handle = tokio::spawn(async move {
                    while let Some(h) = handler.next().await {
                        if h.is_err() {
                            break;
                        }
                    }
                });

                let page = browser.new_page(Self::INDEX_PATH).await?;
                tracing::info!("navigating to page");

                tracing::info!("setting locale and timezone");
                let page = page.emulate_timezone("Australia/Perth").await?;
                let page = page
                    .emulate_locale(SetLocaleOverrideParams::builder().locale("en-AU").build())
                    .await?;
                page.reload().await?;

                tokio::time::sleep(Duration::from_secs(10)).await;

                tracing::info!("screenshot taken");
                let image = page
                    .screenshot(
                        ScreenshotParams::builder()
                            .format(CaptureScreenshotFormat::Png)
                            .full_page(false)
                            .build(),
                    )
                    .await?;

                self.shared_actor_state
                    .object_registry
                    .put_object("home-gateway", "image.png", &image, None)
                    .await?;

                tracing::info!("screenshot uploaded");

                browser.close().await?;
                handle.await?;
            }
        }

        Ok(())
    }
}
