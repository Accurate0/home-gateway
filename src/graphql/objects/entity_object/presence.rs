use async_graphql::Object;
use chrono::{DateTime, Utc};

use crate::{
    actors::{
        devices::presence_sensor::{Message as PresenceMessage, PresenceSensorHandler},
        rpc,
    },
    device_registry::{Capability, DeviceRegistry},
    graphql::objects::entity_object::{QUERY_TIMEOUT, last_seen_for},
};

pub struct PresenceEntity {
    /// machine slug (configured sensor id).
    pub id: String,
    /// human-friendly name.
    pub name: String,
    /// ieee address or esphome node, the RPC key for the presence actor.
    pub address: String,
    pub capabilities: Vec<Capability>,
    pub room: Option<String>,
}

impl PresenceEntity {
    pub fn from_registry(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.presence(address)?;
        let id = registry.id_for_address(address).unwrap_or(address).to_owned();
        let name = if settings.name.is_empty() {
            id.clone()
        } else {
            settings.name.clone()
        };
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
impl PresenceEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Presence
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

    /// Whether presence is detected. Nullable so an unreachable presence actor
    /// reports the error against this field without nulling the whole entity.
    async fn present(&self) -> async_graphql::Result<Option<bool>> {
        let present: Option<bool> =
            rpc::query_factory(PresenceSensorHandler::NAME, QUERY_TIMEOUT, |reply| {
                PresenceMessage::QueryLatest {
                    sensor: self.address.clone(),
                    reply,
                }
            })
            .await?;
        Ok(Some(present.unwrap_or(false)))
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        last_seen_for(ctx, &self.address).await
    }
}
