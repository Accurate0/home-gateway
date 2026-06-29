use crate::auth::AuthContext;
use crate::graphql::objects::auth_object::AuthObject;
use async_graphql::Object;

#[derive(Default)]
pub struct AuthQuery;

#[Object]
impl AuthQuery {
    async fn auth(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<AuthObject> {
        let auth = ctx.data::<AuthContext>()?;

        Ok(AuthObject {
            id: auth.key_id.map(|id| id.to_string()),
            name: auth.name.clone(),
            scopes: auth.scopes.iter().map(ToString::to_string).collect(),
        })
    }
}
