use http::HeaderMap;
use regex::Regex;
use std::{sync::LazyLock, time::Duration};
use tracing::instrument;

use crate::{
    http::wrap_client_in_middleware_no_tracing,
    reddit::types::{RedditFeedResponse, RedditPost},
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
    #[error(transparent)]
    Feed(#[from] quick_xml::DeError),
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
    pub const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36";
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
        let url = format!("{}/r/{subreddit}/top.rss", Self::BASE_URL);
        tracing::info!(
            "fetching top posts from r/{subreddit} over {}",
            timespan.as_str()
        );

        let body = self
            .client
            .get(url)
            .query(&[("t", timespan.as_str()), ("limit", &limit.to_string())])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let feed = quick_xml::de::from_str::<RedditFeedResponse>(&body)?;

        Ok(feed.entries)
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

static LINK_ANCHOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<a href="([^"]+)">\[link\]</a>"#).unwrap());

pub fn image_url(post: &RedditPost) -> Option<String> {
    let content = unescape_html(&post.content.text);
    let url = LINK_ANCHOR
        .captures(&content)
        .and_then(|captures| captures.get(1))?
        .as_str();

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

    fn feed(xml: &str) -> Vec<RedditPost> {
        quick_xml::de::from_str::<RedditFeedResponse>(xml)
            .unwrap()
            .entries
    }

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
    <feed xmlns="http://www.w3.org/2005/Atom">
      <entry>
        <title>A discussion</title>
        <link href="https://www.reddit.com/r/test/comments/2/" />
        <content type="html">&lt;!-- SC_OFF --&gt;&lt;div&gt;some text&lt;/div&gt; &lt;a href="https://www.reddit.com/r/test/comments/2/"&gt;[comments]&lt;/a&gt;</content>
      </entry>
      <entry>
        <title>A mountain</title>
        <link href="https://www.reddit.com/r/test/comments/4/" />
        <content type="html">&lt;a href="https://i.redd.it/mountain.jpg?width=4000&amp;amp;s=abc"&gt;[link]&lt;/a&gt; &lt;a href="https://www.reddit.com/r/test/comments/4/"&gt;[comments]&lt;/a&gt;</content>
      </entry>
      <entry>
        <title>Upper case extension</title>
        <link href="https://www.reddit.com/r/test/comments/5/" />
        <content type="html">&lt;a href="https://i.redd.it/direct.PNG?x=1"&gt;[link]&lt;/a&gt;</content>
      </entry>
      <entry>
        <title>A video</title>
        <link href="https://www.reddit.com/r/test/comments/6/" />
        <content type="html">&lt;a href="https://v.redd.it/clip"&gt;[link]&lt;/a&gt;</content>
      </entry>
    </feed>"#;

    #[test]
    fn feed_parses_entry_titles_and_links() {
        let posts = feed(FIXTURE);

        assert_eq!(posts.len(), 4);
        assert_eq!(posts[0].title, "A discussion");
        assert_eq!(
            posts[0].link.href,
            "https://www.reddit.com/r/test/comments/2/"
        );
    }

    #[test]
    fn image_url_skips_unusable_posts() {
        let posts = feed(FIXTURE);

        assert_eq!(image_url(&posts[0]), None);
        assert_eq!(image_url(&posts[3]), None);
    }

    #[test]
    fn image_url_unescapes_the_link_target() {
        let posts = feed(FIXTURE);

        assert_eq!(
            image_url(&posts[1]).as_deref(),
            Some("https://i.redd.it/mountain.jpg?width=4000&s=abc")
        );
    }

    #[test]
    fn image_url_matches_extensions_case_insensitively() {
        let posts = feed(FIXTURE);

        assert_eq!(
            image_url(&posts[2]).as_deref(),
            Some("https://i.redd.it/direct.PNG?x=1")
        );
    }
}
