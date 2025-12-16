use crate::types::SharedActorState;
use chromiumoxide::{
    Browser, BrowserConfig, browser::HeadlessMode,
    cdp::browser_protocol::page::CaptureScreenshotFormat, handler::viewport::Viewport,
    page::ScreenshotParams,
};
use futures::StreamExt;
use ractor::Actor;
use std::time::Duration;

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
    pub const INDEX_PATH: &str = "file:///app/einkweb/index.html";
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
        let _join_handle = myself.send_after(Duration::from_secs(1), || {
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
                tracing::info!("starting browser");
                let (mut browser, mut handler) = Browser::launch(
                    BrowserConfig::builder()
                        .headless_mode(HeadlessMode::New)
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

                tokio::time::sleep(Duration::from_secs(5)).await;

                tracing::info!("screenshot taken");
                let image = page
                    .screenshot(
                        ScreenshotParams::builder()
                            .format(CaptureScreenshotFormat::Png)
                            .full_page(true)
                            .build(),
                    )
                    .await?;

                let eink_display_bucket = self.shared_actor_state.bucket_accessor.eink_display();
                eink_display_bucket
                    .put_object_builder("/image.png", &image)
                    .with_content_type("image/png")
                    .execute()
                    .await?;
                tracing::info!("screenshot uploaded");

                browser.close().await?;
                handle.await?;
            }
        }

        Ok(())
    }
}
