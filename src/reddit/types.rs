use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RedditFeedResponse {
    #[serde(default, rename = "entry")]
    pub entries: Vec<RedditPost>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditPost {
    pub title: String,
    pub link: RedditLink,
    pub content: RedditContent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditLink {
    #[serde(rename = "@href")]
    pub href: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditContent {
    #[serde(default, rename = "$text")]
    pub text: String,
}
