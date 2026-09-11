use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Deserialize;

/// Declarative API key: config is the source of truth for a key's name + scopes.
/// Secret material is never here — the admin API mints/regenerates the token and
/// startup reconciles these scopes onto the matching DB row by `name`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ApiKeySettings {
    pub name: String,
    pub scopes: Vec<String>,
    #[serde(default)]
    #[schemars(with = "Option<String>")]
    pub expires_at: Option<DateTime<Utc>>,
}
