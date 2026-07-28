use async_graphql::Object;
use chrono::{DateTime, Utc};

use crate::{
    actors::{
        events::door_events::{DerivedDoorEvents, DoorEventsMessage},
        rpc,
    },
    device_registry::{Capability, DeviceRegistry},
    graphql::objects::entity_object::{QUERY_TIMEOUT, last_seen_for},
    types::db::DoorState,
};

pub struct DoorEntity {
    /// configured friendly id, exposed to clients.
    pub id: String,
    pub name: String,
    /// ieee address, the RPC key for the door-events actor.
    pub address: String,
    pub capabilities: Vec<Capability>,
    pub room: Option<String>,
}

impl DoorEntity {
    pub fn from_registry(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.door(address)?;
        Some(Self {
            id: settings.id.clone(),
            name: settings.name.clone(),
            address: address.to_owned(),
            capabilities: registry.capabilities(address).to_vec(),
            room: registry.room(address).map(str::to_owned),
        })
    }
}

#[Object]
impl DoorEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Doors
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

    /// Whether the door is open. Nullable so an unreachable door-events actor
    /// reports the error against this field without nulling the whole entity.
    async fn open(&self) -> async_graphql::Result<Option<bool>> {
        let state: Option<DoorState> =
            rpc::query(DerivedDoorEvents::NAME, QUERY_TIMEOUT, |reply| {
                DoorEventsMessage::QueryState {
                    ieee_addr: self.address.clone(),
                    reply,
                }
            })
            .await?;
        Ok(Some(matches!(state, Some(DoorState::Open))))
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        last_seen_for(ctx, &self.address).await
    }
}
