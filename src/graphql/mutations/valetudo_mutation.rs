use async_graphql::Object;

use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::mqtt::MqttClient;

pub struct ValetudoMutation {
    pub command_topic: String,
    pub start_payload: String,
    pub stop_payload: String,
    pub dock_payload: String,
}

impl ValetudoMutation {
    async fn publish(
        &self,
        ctx: &async_graphql::Context<'_>,
        payload: &str,
    ) -> async_graphql::Result<bool> {
        let mqtt = ctx.data::<MqttClient>()?;
        mqtt.send_event_raw(self.command_topic.clone(), payload)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }
}

#[Object]
impl ValetudoMutation {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_VALETUDO_WRITE))]
    async fn start(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let payload = self.start_payload.clone();
        self.publish(ctx, &payload).await
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_VALETUDO_WRITE))]
    async fn stop(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let payload = self.stop_payload.clone();
        self.publish(ctx, &payload).await
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_VALETUDO_WRITE))]
    async fn dock(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let payload = self.dock_payload.clone();
        self.publish(ctx, &payload).await
    }
}
