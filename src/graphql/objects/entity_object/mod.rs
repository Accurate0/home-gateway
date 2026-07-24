//! Read-only GraphQL view of every stateful entity and its current state.
//!
//! Mirrors the stateful domains of the `EventUpdate` subscription (light, door,
//! presence, environment). Current state is read the same way workflow
//! conditions read it — named-actor RPC (see [`crate::actors::workflows::conditions`])
//! — except environment, which reuses the existing temperature dataloader.

use std::time::Duration;

use async_graphql::{Union, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::graphql::dataloader::last_seen::LastSeenDataLoader;

pub mod door;
pub mod eink_display;
pub mod environment;
pub mod light;
pub mod presence;
pub mod roborock;

pub use door::DoorEntity;
pub use eink_display::{EinkDisplayEntity, EinkDisplayKind};
pub use environment::EnvironmentEntity;
pub use light::LightEntity;
pub use presence::PresenceEntity;
pub use roborock::RoborockEntity;

pub(super) const QUERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Most recent time the device behind `address` was heard from, resolved via the
/// batched `device_last_seen` dataloader. Nullable when the device has never been
/// recorded.
pub(super) async fn last_seen_for(
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
    EinkDisplay(EinkDisplayEntity),
    Roborock(RoborockEntity),
}
