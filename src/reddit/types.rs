use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RedditListingResponse {
    pub data: RedditListing,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditListing {
    pub children: Vec<RedditChild>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditChild {
    pub data: RedditPost,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditPost {
    pub title: String,
    pub permalink: String,
    pub url: Option<String>,
    pub post_hint: Option<String>,
    #[serde(default)]
    pub stickied: bool,
    #[serde(default)]
    pub over_18: bool,
    #[serde(default)]
    pub is_self: bool,
    pub preview: Option<RedditPreview>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditPreview {
    pub images: Vec<RedditPreviewImage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditPreviewImage {
    pub source: RedditPreviewSource,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditPreviewSource {
    pub url: String,
    pub width: u32,
    pub height: u32,
}
