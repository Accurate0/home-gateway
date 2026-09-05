use crate::eink::image::content_hash;
use crate::eink::manager::resolve::ResolvedDisplay;
use crate::eink::manager::source::{ImageSource, Prepared, SourceContext, SourceImage};
use crate::error::AppError;
use crate::integrations::s3::{OptionalObjectResponse, S3};
use crate::settings::EinkMode;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

const INDEX_KEY: &str = "eink-display/web/index.html";
const DEFAULT_VIEW: &str = "default";

#[cfg(not(debug_assertions))]
pub const INDEX_PATH: &str = "file:///tmp/index.html";
#[cfg(debug_assertions)]
pub const INDEX_PATH: &str =
    "file:///home/anurag/Projects/home-gateway/eink-display-web/dist/index.html";
pub const INDEX_FS_PATH: &str = "/tmp/index.html";

#[derive(Default)]
pub struct DashboardSource {
    index_etag: Mutex<Option<String>>,
}

impl DashboardSource {
    pub fn key(view_name: &str, orientation: &str) -> String {
        format!("eink-display/dashboard/{view_name}-{orientation}.png")
    }

    pub fn key_for(display: &ResolvedDisplay) -> String {
        Self::key(Self::view_name(display), display.orientation_str())
    }

    fn view_name(display: &ResolvedDisplay) -> &str {
        display
            .view
            .as_ref()
            .map_or(DEFAULT_VIEW, |view| view.name.as_str())
    }

    fn url(display: &ResolvedDisplay) -> String {
        let orientation = format!("orientation={}", display.orientation_str());
        let query = match display.view.as_ref().and_then(|view| view.query.as_deref()) {
            Some(query) if !query.is_empty() => format!("{query}&{orientation}"),
            _ => orientation,
        };

        format!("{INDEX_PATH}?{query}")
    }

    async fn save_index_if_new(&self, s3: &S3) -> Result<(), AppError> {
        let mut etag = self.index_etag.lock().await;

        match s3.get_object_optional(INDEX_KEY, etag.as_deref()).await? {
            OptionalObjectResponse::ExistingObjectIsValid => {
                tracing::info!("304 from server, not replacing file");
            }
            OptionalObjectResponse::ObjectUpdated(object) => {
                tracing::info!("index fetched: etag {:?}", object.etag);

                let mut file = tokio::fs::File::create(INDEX_FS_PATH).await?;
                file.write_all(&object.payload).await?;
                *etag = Some(object.etag);
            }
        }

        Ok(())
    }

    async fn stored_image_key(
        eink: &crate::repo::EinkRepo,
        device_id: &str,
    ) -> Result<Option<String>, AppError> {
        let stored = eink.stored_render(device_id).await?;

        Ok(stored.and_then(|row| row.image_key))
    }
}

#[async_trait::async_trait]
impl ImageSource for DashboardSource {
    fn name(&self) -> &'static str {
        "dashboard"
    }

    fn mode(&self) -> EinkMode {
        EinkMode::Dashboard
    }

    async fn acquire(&self, ctx: &SourceContext<'_>) -> Result<Option<SourceImage>, AppError> {
        if !cfg!(debug_assertions) {
            self.save_index_if_new(ctx.s3).await?;
        }

        let image = ctx
            .screenshots
            .screenshot(
                &Self::url(ctx.display),
                ctx.display.target_dims(),
                ctx.display.settle,
            )
            .await?;

        let Some(image) = image else {
            return Ok(None);
        };

        let image_key = Self::key_for(ctx.display);
        ctx.s3
            .put_object(&image_key, &image, Some("image/png"))
            .await?;

        Ok(Some(SourceImage {
            content_hash: content_hash(&image),
            image_key,
        }))
    }

    async fn prepare(&self, ctx: &SourceContext<'_>) -> Result<Prepared, AppError> {
        let image_key = Self::key_for(ctx.display);

        if Self::stored_image_key(ctx.eink, &ctx.display.device_id).await?
            == Some(image_key.clone())
        {
            return Ok(Prepared::Skip);
        }

        let image = match ctx.s3.get_object(&image_key).await {
            Ok(image) => image,
            Err(e) => {
                tracing::warn!(
                    "no previous dashboard render at {image_key} for {} ({e}), \
                     serving the current frame until the screenshot lands",
                    ctx.display.device_id
                );
                return Ok(Prepared::NeedsRefresh);
            }
        };

        Ok(Prepared::Ready(SourceImage {
            content_hash: content_hash(&image),
            image_key,
        }))
    }
}
