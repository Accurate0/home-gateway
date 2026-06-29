use async_graphql::{Context, Guard, Result};

use crate::auth::{AuthContext, scope::Scope};

pub struct ScopeGuard(pub Scope);

impl Guard for ScopeGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let auth = ctx.data::<AuthContext>()?;
        if auth.has(&self.0) {
            Ok(())
        } else {
            Err("insufficient scope".into())
        }
    }
}
