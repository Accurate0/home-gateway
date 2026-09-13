use std::sync::Arc;

use crate::auth::AuthContext;
use crate::auth::scope::{Action, Resource, Scope};

#[derive(Clone, Default)]
pub enum LuaAuthority {
    #[default]
    Trusted,
    Delegated(Arc<AuthContext>),
}

impl LuaAuthority {
    pub fn delegated(auth: AuthContext) -> Self {
        LuaAuthority::Delegated(Arc::new(auth))
    }

    pub fn allows(&self, resource: Resource, action: Action) -> bool {
        match self {
            LuaAuthority::Trusted => true,
            LuaAuthority::Delegated(auth) => auth.has(&Scope::new(resource, action)),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            LuaAuthority::Trusted => "trusted".to_owned(),
            LuaAuthority::Delegated(auth) => {
                auth.name.clone().unwrap_or_else(|| "delegated".to_owned())
            }
        }
    }
}
