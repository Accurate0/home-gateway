//! Read-only GraphQL view of every stateful entity and its current state.
//!
//! Mirrors the stateful domains of the `EventUpdate` subscription (light, door,
//! presence, environment). Current state is read the same way workflow
//! conditions read it — named-actor RPC (see [`crate::actors::workflows::conditions`])
//! — except environment, which reuses the existing temperature dataloader.

use std::time::Duration;

use async_graphql::{Enum, SimpleObject, Union, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::graphql::dataloader::last_seen::LastSeenDataLoader;

pub mod door;
pub mod eink_display;
pub mod environment;
pub mod light;
pub mod presence;
pub mod robot_vacuum;

pub use door::DoorEntity;
pub use eink_display::EinkDisplayEntity;
pub use environment::EnvironmentEntity;
pub use light::LightEntity;
pub use presence::PresenceEntity;
pub use robot_vacuum::RobotVacuumEntity;

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

/// The dashboard section a device kind is displayed under. Owned by the backend
/// so categorising a kind never requires a frontend change. Variants are declared
/// in display order — see [`EntityCategory::ORDERED`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum EntityCategory {
    Lights,
    Doors,
    Presence,
    Environment,
    Displays,
    Vacuums,
}

impl EntityCategory {
    /// Every category in the order sections should render.
    pub const ORDERED: [EntityCategory; 6] = [
        EntityCategory::Lights,
        EntityCategory::Doors,
        EntityCategory::Presence,
        EntityCategory::Environment,
        EntityCategory::Displays,
        EntityCategory::Vacuums,
    ];
}

impl std::fmt::Display for EntityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            EntityCategory::Lights => "Lights",
            EntityCategory::Doors => "Doors",
            EntityCategory::Presence => "Presence",
            EntityCategory::Environment => "Environment",
            EntityCategory::Displays => "Displays",
            EntityCategory::Vacuums => "Vacuums",
        })
    }
}

#[derive(SimpleObject)]
pub struct EntitySection {
    pub category: EntityCategory,
    pub title: String,
}

impl EntitySection {
    pub fn ordered() -> Vec<EntitySection> {
        EntityCategory::ORDERED
            .into_iter()
            .map(|category| EntitySection {
                category,
                title: category.to_string(),
            })
            .collect()
    }
}

#[derive(Union)]
pub enum Entity {
    Light(LightEntity),
    Environment(EnvironmentEntity),
    Door(DoorEntity),
    Presence(PresenceEntity),
    EinkDisplay(EinkDisplayEntity),
    RobotVacuum(RobotVacuumEntity),
}
