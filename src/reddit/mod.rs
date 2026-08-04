use http::HeaderMap;
use std::time::Duration;
use tracing::instrument;

use crate::{
    http::wrap_client_in_middleware_no_tracing,
    reddit::types::{RedditListingResponse, RedditPost},
    settings::RedditTimespan,
};

pub mod types;

pub struct Reddit {
    client: reqwest_middleware::ClientWithMiddleware,
}

#[derive(thiserror::Error, Debug)]
pub enum RedditError {
    #[error(transparent)]
    HttpMiddleware(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("unsupported image content type: {0}")]
    ContentType(String),
    #[error("image too large: {0} bytes")]
    TooLarge(usize),
}

impl Default for Reddit {
    fn default() -> Self {
        Self::new()
    }
}

impl Reddit {
    const BASE_URL: &str = "https://www.reddit.com";
    const USER_AGENT: &str =
        "home-gateway/1.0 (eink display; +https://github.com/anuraaga/home-gateway)";
    const TIMEOUT: Duration = Duration::from_secs(10);
    const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;

    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::USER_AGENT, Self::USER_AGENT.parse().unwrap());

        Self {
            client: wrap_client_in_middleware_no_tracing(
                reqwest::ClientBuilder::new()
                    .default_headers(headers)
                    .timeout(Self::TIMEOUT)
                    .build()
                    .unwrap(),
            )
            .unwrap(),
        }
    }

    #[instrument(skip(self))]
    pub async fn top_posts(
        &self,
        subreddit: &str,
        timespan: RedditTimespan,
        limit: u32,
    ) -> Result<Vec<RedditPost>, RedditError> {
        let url = format!("{}/r/{subreddit}/top.json", Self::BASE_URL);
        tracing::info!(
            "fetching top posts from r/{subreddit} over {}",
            timespan.as_str()
        );

        let resp = self
            .client
            .get(url)
            .query(&[("t", timespan.as_str()), ("limit", &limit.to_string())])
            .send()
            .await?
            .error_for_status()?
            .json::<RedditListingResponse>()
            .await?;

        Ok(resp.data.children.into_iter().map(|c| c.data).collect())
    }

    #[instrument(skip(self))]
    pub async fn download_image(&self, url: &str) -> Result<Vec<u8>, RedditError> {
        let resp = self.client.get(url).send().await?.error_for_status()?;

        let content_type = resp
            .headers()
            .get(http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        if !content_type.starts_with("image/") {
            return Err(RedditError::ContentType(content_type));
        }

        let bytes = resp.bytes().await?;
        if bytes.len() > Self::MAX_IMAGE_BYTES {
            return Err(RedditError::TooLarge(bytes.len()));
        }

        Ok(bytes.to_vec())
    }
}

const IMAGE_EXTENSIONS: [&str; 4] = [".jpg", ".jpeg", ".png", ".webp"];

pub fn image_url(post: &RedditPost) -> Option<String> {
    if post.stickied || post.over_18 || post.is_self {
        return None;
    }

    let preview = post
        .preview
        .as_ref()
        .and_then(|preview| preview.images.first())
        .map(|image| unescape_html(&image.source.url));
    if let Some(preview) = preview {
        return Some(preview);
    }

    let url = post.url.as_deref()?;
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();

    IMAGE_EXTENSIONS
        .iter()
        .any(|ext| path.ends_with(ext))
        .then(|| url.to_owned())
}

fn unescape_html(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(json: &str) -> Vec<RedditPost> {
        serde_json::from_str::<RedditListingResponse>(json)
            .unwrap()
            .data
            .children
            .into_iter()
            .map(|c| c.data)
            .collect()
    }

    const FIXTURE: &str = r#"{
      "data": { "children": [
        { "data": {
            "title": "Announcement",
            "permalink": "/r/test/1",
            "url": "https://reddit.com/r/test",
            "stickied": true,
            "preview": { "images": [ { "source": { "url": "https://preview.redd.it/a.jpg", "width": 10, "height": 10 } } ] }
        } },
        { "data": {
            "title": "A discussion",
            "permalink": "/r/test/2",
            "url": "https://reddit.com/r/test/2",
            "is_self": true
        } },
        { "data": {
            "title": "Nsfw",
            "permalink": "/r/test/3",
            "url": "https://i.redd.it/nsfw.jpg",
            "over_18": true
        } },
        { "data": {
            "title": "A mountain",
            "permalink": "/r/test/4",
            "url": "https://i.redd.it/mountain.jpg",
            "post_hint": "image",
            "preview": { "images": [ { "source": { "url": "https://preview.redd.it/m.jpg?width=4000&amp;s=abc", "width": 4000, "height": 3000 } } ] }
        } },
        { "data": {
            "title": "Direct link only",
            "permalink": "/r/test/5",
            "url": "https://i.redd.it/direct.PNG?x=1"
        } },
        { "data": {
            "title": "A video",
            "permalink": "/r/test/6",
            "url": "https://v.redd.it/clip"
        } }
      ] }
    }"#;

    #[test]
    fn image_url_skips_unusable_posts() {
        let posts = listing(FIXTURE);

        assert_eq!(image_url(&posts[0]), None);
        assert_eq!(image_url(&posts[1]), None);
        assert_eq!(image_url(&posts[2]), None);
        assert_eq!(image_url(&posts[5]), None);
    }

    #[test]
    fn image_url_prefers_the_unescaped_preview_source() {
        let posts = listing(FIXTURE);

        assert_eq!(
            image_url(&posts[3]).as_deref(),
            Some("https://preview.redd.it/m.jpg?width=4000&s=abc")
        );
    }

    #[test]
    fn image_url_falls_back_to_a_direct_image_link() {
        let posts = listing(FIXTURE);

        assert_eq!(
            image_url(&posts[4]).as_deref(),
            Some("https://i.redd.it/direct.PNG?x=1")
        );
    }
}
