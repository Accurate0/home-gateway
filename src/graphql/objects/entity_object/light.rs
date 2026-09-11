use async_graphql::{Object, SimpleObject};
use chrono::{DateTime, Utc};

use crate::{
    actors::{
        devices::light::{LightHandler, LightHandlerMessage},
        system::rpc,
    },
    device_registry::{Capability, DeviceRegistry},
    graphql::objects::entity_object::{last_seen_for, query_timeout},
    repo::{
        RepoRegistry,
        intent::{DeviceIntent, DeviceKind},
        light::LightState,
    },
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

    async fn state(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<&LightState> {
        let timeout = query_timeout(ctx)?;

        self.state
            .get_or_try_init(|| async {
                rpc::query_factory(LightHandler::NAME, timeout, |reply| {
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
    async fn on(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<bool>> {
        Ok(Some(self.state(ctx).await?.on))
    }

    /// Current brightness, 0-254. Null when the light has not reported one.
    async fn brightness(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<i32>> {
        Ok(self.state(ctx).await?.brightness)
    }

    /// Colour temperature in mireds (1000000/kelvin): 153 is coolest, 500 warmest.
    async fn colour_temperature(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<i32>> {
        Ok(self.state(ctx).await?.colour_temp)
    }

    /// Current colour as `#rrggbb`. Null when the light has not reported one.
    async fn colour(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<&str>> {
        Ok(self.state(ctx).await?.colour.as_deref())
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        last_seen_for(ctx, &self.address).await
    }

    async fn pending_commands(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<PendingCommand>> {
        let pending = ctx
            .data::<RepoRegistry>()?
            .intent()
            .pending_for(DeviceKind::Light, &self.address)
            .await?;

        Ok(pending
            .iter()
            .filter_map(PendingCommand::of_light)
            .collect())
    }
}

#[derive(SimpleObject)]
pub struct PendingCommand {
    pub requested_at: DateTime<Utc>,
    pub attempts: i32,
    pub relative: bool,
    pub state: Option<String>,
    pub brightness: Option<i32>,
    pub colour_temperature: Option<i32>,
    pub colour: Option<String>,
}

impl PendingCommand {
    fn of_light(intent: &DeviceIntent) -> Option<Self> {
        let attributes = intent.attributes.as_light()?;

        Some(Self {
            requested_at: intent.requested_at,
            attempts: intent.attempts,
            relative: intent.relative,
            state: attributes.state.clone(),
            brightness: attributes.brightness,
            colour_temperature: attributes.colour_temp,
            colour: attributes.colour.clone(),
        })
    }
}
