use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;

use super::OAuthCacheSettings;

/// Default for the OAuth `groups` claim name.
fn default_groups_claim() -> String {
    "groups".to_owned()
}

/// OAuth (OIDC) settings. Access tokens are JWTs validated locally against the
/// provider's JWKS — no client secret is needed. A caller's scopes are derived
/// from their group memberships (the `groups_claim`) via `group_scopes`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct OAuthSettings {
    pub issuer: String,
    pub jwks_url: String,
    pub userinfo_url: String,
    pub audience: String,
    #[serde(default = "default_groups_claim")]
    pub groups_claim: String,
    /// group SPN -> granted scope patterns (`resource.path:action`).
    pub group_scopes: HashMap<String, Vec<String>>,
    pub cache: OAuthCacheSettings,
}
