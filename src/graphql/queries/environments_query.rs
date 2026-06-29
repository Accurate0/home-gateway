use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::environment_object::EnvironmentObject;
use async_graphql::Object;

#[derive(Default)]
pub struct EnvironmentsQuery;

#[Object]
impl EnvironmentsQuery {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_ENTITY_READ))]
    async fn environment(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnvironmentObject> {
        Ok(EnvironmentObject {})
    }
}
