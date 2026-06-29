use http::StatusCode;
use uuid::Uuid;

use super::scope::{Scope, ScopePattern};

#[derive(Debug, Clone)]
pub struct AuthContext {
    #[allow(dead_code)]
    pub key_id: Option<Uuid>,
    pub scopes: Vec<ScopePattern>,
    #[allow(dead_code)]
    pub legacy: bool,
}

impl AuthContext {
    pub fn full_access(legacy: bool) -> Self {
        Self {
            key_id: None,
            scopes: vec![ScopePattern::Global],
            legacy,
        }
    }

    pub fn from_scopes(key_id: Option<Uuid>, scopes: &[String]) -> Self {
        let scopes = scopes
            .iter()
            .filter_map(|raw| match ScopePattern::parse(raw) {
                Some(pattern) => Some(pattern),
                None => {
                    tracing::warn!("ignoring invalid scope: {raw}");
                    None
                }
            })
            .collect();

        Self {
            key_id,
            scopes,
            legacy: false,
        }
    }

    pub fn has(&self, required: &Scope) -> bool {
        self.scopes.iter().any(|s| s.matches(required))
    }

    pub fn require(&self, required: &Scope) -> Result<(), StatusCode> {
        if self.has(required) {
            Ok(())
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
}
