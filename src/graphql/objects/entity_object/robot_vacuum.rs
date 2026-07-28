use async_graphql::{Enum, Object, dataloader::DataLoader};
use chrono::{DateTime, Utc};

use crate::device_registry::{Capability, DeviceRegistry};
use crate::graphql::dataloader::home_assistant_state::HomeAssistantStateDataLoader;
use crate::graphql::dataloader::valetudo_state::{ValetudoStateDataLoader, ValetudoStateModel};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum RobotVacuumKind {
    Roborock,
    Valetudo,
}

enum Source {
    Roborock {
        status_entity: String,
        battery_entity: String,
        room_entity: String,
    },
    Valetudo {
        identifier: String,
    },
}

pub struct RobotVacuumEntity {
    pub id: String,
    pub name: String,
    pub room: Option<String>,
    pub capabilities: Vec<Capability>,
    source: Source,
}

impl RobotVacuumEntity {
    pub fn from_roborock(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.roborock(address)?;
        let id = registry.id_for_address(address).unwrap_or(address).to_owned();
        Some(Self {
            id,
            name: settings.name.clone(),
            room: registry.room(address).map(str::to_owned),
            capabilities: registry.capabilities(address).to_vec(),
            source: Source::Roborock {
                status_entity: settings.status_entity.clone(),
                battery_entity: settings.battery_entity.clone(),
                room_entity: settings.room_entity.clone(),
            },
        })
    }

    pub fn from_valetudo(registry: &DeviceRegistry, address: &str) -> Option<Self> {
        let settings = registry.valetudo(address)?;
        let id = registry.id_for_address(address).unwrap_or(address).to_owned();
        Some(Self {
            id,
            name: settings.name.clone(),
            room: registry.room(address).map(str::to_owned),
            capabilities: registry.capabilities(address).to_vec(),
            source: Source::Valetudo {
                identifier: address.to_owned(),
            },
        })
    }

    async fn ha_state(
        &self,
        ctx: &async_graphql::Context<'_>,
        entity_id: &str,
    ) -> async_graphql::Result<Option<String>> {
        let loader = ctx.data::<DataLoader<HomeAssistantStateDataLoader>>()?;
        Ok(loader.load_one(entity_id.to_owned()).await?.map(|s| s.state))
    }

    async fn valetudo_model(
        &self,
        ctx: &async_graphql::Context<'_>,
        identifier: &str,
    ) -> async_graphql::Result<Option<ValetudoStateModel>> {
        let loader = ctx.data::<DataLoader<ValetudoStateDataLoader>>()?;
        Ok(loader.load_one(identifier.to_owned()).await?)
    }
}

#[Object]
impl RobotVacuumEntity {
    async fn category(&self) -> super::EntityCategory {
        super::EntityCategory::Vacuums
    }

    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn room(&self) -> Option<&str> {
        self.room.as_deref()
    }

    async fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    async fn kind(&self) -> RobotVacuumKind {
        match self.source {
            Source::Roborock { .. } => RobotVacuumKind::Roborock,
            Source::Valetudo { .. } => RobotVacuumKind::Valetudo,
        }
    }

    async fn status(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        match &self.source {
            Source::Roborock { status_entity, .. } => self.ha_state(ctx, status_entity).await,
            Source::Valetudo { identifier } => {
                Ok(self.valetudo_model(ctx, identifier).await?.and_then(|m| m.state))
            }
        }
    }

    async fn battery_percentage(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        match &self.source {
            Source::Roborock { battery_entity, .. } => Ok(self
                .ha_state(ctx, battery_entity)
                .await?
                .and_then(|s| s.parse::<f64>().ok())),
            Source::Valetudo { identifier } => Ok(self
                .valetudo_model(ctx, identifier)
                .await?
                .and_then(|m| m.battery_level)
                .map(|b| b as f64)),
        }
    }

    async fn current_room(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        match &self.source {
            Source::Roborock { room_entity, .. } => self.ha_state(ctx, room_entity).await,
            Source::Valetudo { .. } => Ok(None),
        }
    }

    async fn fan_speed(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        match &self.source {
            Source::Roborock { .. } => Ok(None),
            Source::Valetudo { identifier } => {
                Ok(self.valetudo_model(ctx, identifier).await?.and_then(|m| m.fan_speed))
            }
        }
    }

    async fn current_clean_area(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        match &self.source {
            Source::Roborock { .. } => Ok(None),
            Source::Valetudo { identifier } => Ok(self
                .valetudo_model(ctx, identifier)
                .await?
                .and_then(|m| m.current_clean_area)),
        }
    }

    async fn clean_count(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<i32>> {
        match &self.source {
            Source::Roborock { .. } => Ok(None),
            Source::Valetudo { identifier } => {
                Ok(self.valetudo_model(ctx, identifier).await?.and_then(|m| m.clean_count))
            }
        }
    }

    async fn last_seen(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<DateTime<Utc>>> {
        match &self.source {
            Source::Roborock {
                status_entity,
                battery_entity,
                room_entity,
            } => {
                let loader = ctx.data::<DataLoader<HomeAssistantStateDataLoader>>()?;
                let entities = [
                    status_entity.clone(),
                    battery_entity.clone(),
                    room_entity.clone(),
                ];
                let mut latest: Option<DateTime<Utc>> = None;
                for entity_id in entities {
                    if let Some(model) = loader.load_one(entity_id).await? {
                        latest =
                            Some(latest.map_or(model.updated_at, |cur| cur.max(model.updated_at)));
                    }
                }
                Ok(latest)
            }
            Source::Valetudo { identifier } => {
                Ok(self.valetudo_model(ctx, identifier).await?.map(|m| m.updated_at))
            }
        }
    }
}
