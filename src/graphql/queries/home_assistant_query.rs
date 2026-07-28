use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::graphql::objects::home_assistant_object::HomeAssistantObject;
use async_graphql::Object;

#[derive(Default)]
pub struct HomeAssistantQuery;

#[Object]
impl HomeAssistantQuery {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_HOME_ASSISTANT_READ))]
    async fn home_assistant(
        &self,
        _ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<HomeAssistantObject> {
        Ok(HomeAssistantObject {})
    }
}
