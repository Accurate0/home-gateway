use crate::graphql::objects::environment_object::EnvironmentObject;
use async_graphql::Object;

#[derive(Default)]
pub struct EnvironmentsQuery;

#[Object]
impl EnvironmentsQuery {
    async fn environment(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<EnvironmentObject> {
        Ok(EnvironmentObject {})
    }
}
