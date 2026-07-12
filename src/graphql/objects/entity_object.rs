//! Read-only GraphQL view of every stateful entity and its current state.
//!
//! Mirrors the stateful domains of the `EventUpdate` subscription (light, door,
//! presence, environment). Current state is read the same way workflow
//! conditions read it — named-actor RPC (see [`crate::actors::workflows::conditions`])
//! — except environment, which reuses the existing temperature dataloader.

use std::time::Duration;

use async_graphql::{Object, Union, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::{
    actors::{
        devices::presence_sensor::{Message as PresenceMessage, PresenceSensorHandler},
        events::door_events::{DerivedDoorEvents, DoorEventsMessage},
        light::{LightHandler, LightHandlerMessage},
        rpc,
    },
    device_registry::Capability,
    graphql::dataloader::{
        last_seen::LastSeenDataLoader,
        temperature::{LatestTemperatureDataLoader, TemperatureModel},
    },
    types::db::DoorState,
};

const QUERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Most recent time the device behind `address` was heard from, resolved via the
/// batched `device_last_seen` dataloader. Nullable when the device has never been
/// recorded.
async fn last_seen_for(
    ctx: &async_graphql::Context<'_>,
    address: &str,
) -> async_graphql::Result<Option<DateTime<Utc>>> {
    let loader = ctx.data::<DataLoader<LastSeenDataLoader>>()?;
    Ok(loader.load_one(address.to_owned()).await?)
}

#[derive(Union)]
pub enum Entity {
    Light(LightEntity),
    Environment(EnvironmentEntity),
    Door(DoorEntity),
    Presence(PresenceEntity),
}

pub struct LightEntity {
    /// machine slug (configured sensor id).
    pub id: String,
    /// human-friendly name.
    pub name: String,
    /// ieee address, the RPC key for the light actor.
    pub address: String,
    pub capabilities: Vec<Capability>,
}

#[Object]
impl LightEntity {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn capabilities(&self) -> &[Capability] {
        &self.capabilities
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

pub struct DoorEntity {
    /// configured friendly id, exposed to clients.
    pub id: String,
    pub name: String,
    /// ieee address, the RPC key for the door-events actor.
    pub address: String,
    pub capabilities: Vec<Capability>,
}

#[Object]
impl DoorEntity {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn capabilities(&self) -> &[Capability] {
        &self.capabilities
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

pub struct PresenceEntity {
    /// machine slug (configured sensor id).
    pub id: String,
    /// human-friendly name.
    pub name: String,
    /// ieee address or esphome node, the RPC key for the presence actor.
    pub address: String,
    pub capabilities: Vec<Capability>,
}

#[Object]
impl PresenceEntity {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn capabilities(&self) -> &[Capability] {
        &self.capabilities
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

pub struct EnvironmentEntity {
    /// configured entity id (e.g. `outdoor`), the dataloader key.
    pub id: String,
    /// human-friendly name.
    pub name: String,
    /// device address (ieee addr or esphome node), the last-seen dataloader key.
    pub address: String,
    pub capabilities: Vec<Capability>,
}

impl EnvironmentEntity {
    async fn load<T, F>(
        &self,
        context: &async_graphql::Context<'_>,
        mapping: F,
    ) -> async_graphql::Result<T>
    where
        F: Fn(TemperatureModel) -> T,
    {
        let loader = context.data::<DataLoader<LatestTemperatureDataLoader>>()?;
        loader
            .load_one(self.id.clone())
            .await?
            .map(mapping)
            .ok_or(anyhow::Error::msg("no details found for this id").into())
    }
}

#[Object]
impl EnvironmentEntity {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    /// Nullable so a missing reading reports the error against this field
    /// without nulling the whole entity.
    async fn temperature(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        Ok(Some(self.load(ctx, |t| t.temperature).await?))
    }

    async fn humidity(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        self.load(ctx, |t| t.humidity).await
    }

    async fn pressure(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        self.load(ctx, |t| t.pressure).await
    }

    async fn lux(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<f64>> {
        self.load(ctx, |t| t.lux).await
    }

    async fn uv_index(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        self.load(ctx, |t| t.uv_index).await
    }

    /// Nullable so a missing reading reports the error against this field
    /// without nulling the whole entity.
    async fn time(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        Ok(Some(self.load(ctx, |t| t.time).await?))
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        last_seen_for(ctx, &self.address).await
    }
}
