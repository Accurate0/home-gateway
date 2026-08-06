use async_graphql::Object;
use chrono::{DateTime, Utc};

use crate::{
    actors::{
        devices::light::{LightHandler, LightHandlerMessage},
        rpc,
    },
    device_registry::{Capability, DeviceRegistry},
    graphql::objects::entity_object::{QUERY_TIMEOUT, last_seen_for},
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
        })
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
        Ok(Some(
            rpc::query_factory(LightHandler::NAME, QUERY_TIMEOUT, |reply| {
                LightHandlerMessage::QueryPowerState {
                    ieee_addr: self.address.clone(),
                    reply,
                }
            })
            .await?,
        ))
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        last_seen_for(ctx, &self.address).await
    }
}
