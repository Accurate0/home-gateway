use async_graphql::Object;
use serde_json::json;

use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::home_assistant::HomeAssistant;

pub struct RoborockMutation {
    pub control_entity: String,
    pub stop_service: String,
    pub dock_service: String,
}

impl RoborockMutation {
    async fn call(
        &self,
        ctx: &async_graphql::Context<'_>,
        service: &str,
    ) -> async_graphql::Result<bool> {
        let home_assistant = ctx.data::<Option<HomeAssistant>>()?;
        let Some(home_assistant) = home_assistant else {
            return Err(async_graphql::Error::new("home assistant is not configured"));
        };

        let Some((domain, service)) = service.split_once('.') else {
            return Err(async_graphql::Error::new(format!(
                "invalid service `{service}`, expected `domain.service`"
            )));
        };

        home_assistant
            .call_service(domain, service, json!({ "entity_id": self.control_entity }))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(true)
    }
}

#[Object]
impl RoborockMutation {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_ROBOROCK_WRITE))]
    async fn stop(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let service = self.stop_service.clone();
        self.call(ctx, &service).await
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_ROBOROCK_WRITE))]
    async fn dock(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let service = self.dock_service.clone();
        self.call(ctx, &service).await
    }
}
