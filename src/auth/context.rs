use uuid::Uuid;

use super::MissingScope;
use super::scope::{Action, Resource, Scope, ScopePattern};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub key_id: Option<Uuid>,
    pub name: Option<String>,
    pub scopes: Vec<ScopePattern>,
}

impl AuthContext {
    pub fn full_access() -> Self {
        Self {
            key_id: None,
            name: None,
            scopes: vec![ScopePattern::parse("**:*").expect("global pattern is valid")],
        }
    }

    pub fn from_scopes(key_id: Option<Uuid>, name: Option<String>, scopes: &[String]) -> Self {
        let scopes = scopes
            .iter()
            .filter_map(|raw| match ScopePattern::parse(raw) {
                Ok(pattern) => Some(pattern),
                Err(e) => {
                    tracing::warn!("ignoring invalid scope '{raw}': {e}");
                    None
                }
            })
            .collect();

        Self {
            key_id,
            name,
            scopes,
        }
    }

    pub fn has(&self, required: &Scope) -> bool {
        self.scopes.iter().any(|s| s.matches(required))
    }

    pub fn require(&self, resource: Resource, action: Action) -> Result<(), MissingScope> {
        let scope = Scope::new(resource, action);

        if self.has(&scope) {
            Ok(())
        } else {
            Err(MissingScope { scope })
        }
    }
}
