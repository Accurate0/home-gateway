use async_graphql::Object;

use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::unraid_object::UnraidObject;
use crate::repo::RepoRegistry;

#[derive(Default)]
pub struct UnraidQuery;

#[Object]
impl UnraidQuery {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Unraid, Action::Read)))]
    async fn unraid(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<UnraidObject>> {
        let latest = ctx.data::<RepoRegistry>()?.unraid().latest().await?;

        Ok(latest.map(UnraidObject))
    }
}
