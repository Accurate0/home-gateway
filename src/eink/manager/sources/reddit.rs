use super::cache::cache_processed_image;
use crate::eink::image::content_hash;
use crate::eink::manager::source::{ImageSource, SourceContext, SourceImage};
use crate::error::AppError;
use crate::integrations::reddit::image_url;
use crate::settings::EinkMode;
use rand::seq::IndexedRandom;

pub struct RedditSource;

#[async_trait::async_trait]
impl ImageSource for RedditSource {
    fn name(&self) -> &'static str {
        "reddit"
    }

    fn mode(&self) -> EinkMode {
        EinkMode::Reddit
    }

    async fn acquire(&self, ctx: &SourceContext<'_>) -> Result<Option<SourceImage>, AppError> {
        let Some(feed) = &ctx.display.feed else {
            tracing::warn!(
                "reddit mode selected for `{}` but no subreddit is configured, skipping render",
                ctx.display.name
            );
            return Ok(None);
        };

        let posts = ctx
            .reddit
            .top_posts(&feed.subreddit, feed.timespan, feed.limit)
            .await?;

        let candidates: Vec<String> = posts.iter().filter_map(image_url).collect();
        let Some(url) = candidates.choose(&mut rand::rng()).cloned() else {
            tracing::warn!(
                "no usable image posts in r/{} over {}, skipping render",
                feed.subreddit,
                feed.timespan.as_str()
            );
            return Ok(None);
        };

        tracing::info!("selected reddit image {url} from r/{}", feed.subreddit);

        let source = ctx.reddit.download_image(&url).await?;
        let content_hash = content_hash(&source);

        let image_key =
            cache_processed_image(ctx.s3, ctx.display, source, &content_hash, &url).await?;

        Ok(Some(SourceImage {
            image_key,
            content_hash,
        }))
    }
}
