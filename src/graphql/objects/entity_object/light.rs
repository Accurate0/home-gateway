use async_graphql::Object;
use chrono::{DateTime, Utc};

use crate::{
    actors::{
        devices::light::{LightHandler, LightHandlerMessage},
        system::rpc,
    },
    device_registry::{Capability, DeviceRegistry},
    graphql::objects::entity_object::{QUERY_TIMEOUT, last_seen_for},
    repo::light::LightState,
};

pub struct LightEntity {
    /// machine slug (configured sensor id).
    pub id: String,
    /// human-friendly name.
    pub name: String,
    /// ieee address, the RPC key for the light actor.
    pub address: String,
    pub capabilities: Vec<Capability>,
    pub room: Option<String>,
    state: tokio::sync::OnceCell<LightState>,
}

impl LightEntity {
    pub fn from_registry(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let name = registry.light(address)?.clone();
        let id = registry
            .id_for_address(address)
            .unwrap_or(address)
            .to_owned();
        Some(Self {
            id,
            name,
            address: address.to_owned(),
            capabilities: registry.capabilities(address).to_vec(),
            room: registry.room(address).map(str::to_owned),
            state: tokio::sync::OnceCell::new(),
        })
    }

    async fn state(&self) -> async_graphql::Result<&LightState> {
        self.state
            .get_or_try_init(|| async {
                rpc::query_factory(LightHandler::NAME, QUERY_TIMEOUT, |reply| {
                    LightHandlerMessage::QueryState {
                        ieee_addr: self.address.clone(),
                        reply,
                    }
                })
                .await
                .map_err(async_graphql::Error::from)
            })
            .await
    }
}

#[Object]
impl LightEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Lights
    }

    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    async fn room(&self) -> Option<&str> {
        self.room.as_deref()
    }

    async fn battery(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<super::DeviceBattery>> {
        super::battery_for(ctx, &self.id).await
    }

    /// Current power state. Nullable so an unreachable light actor reports the
    /// error against this field without nulling the whole entity.
    async fn on(&self) -> async_graphql::Result<Option<bool>> {
        Ok(Some(self.state().await?.on))
    }

    /// Current brightness, 0-254. Null when the light has not reported one.
    async fn brightness(&self) -> async_graphql::Result<Option<i32>> {
        Ok(self.state().await?.brightness)
    }

    /// Colour temperature in mireds (1000000/kelvin): 153 is coolest, 500 warmest.
    async fn colour_temperature(&self) -> async_graphql::Result<Option<i32>> {
        Ok(self.state().await?.colour_temp)
    }

    /// Current colour as `#rrggbb`. Null when the light has not reported one.
    async fn colour(&self) -> async_graphql::Result<Option<&str>> {
        Ok(self.state().await?.colour.as_deref())
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        last_seen_for(ctx, &self.address).await
    }
}
