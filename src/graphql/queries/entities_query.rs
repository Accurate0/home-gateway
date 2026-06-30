use std::sync::Arc;

use async_graphql::Object;

use crate::auth::context::AuthContext;
use crate::auth::scope::required;
use crate::device_registry::DeviceRegistry;
use crate::graphql::objects::entity_object::{
    DoorEntity, Entity, EnvironmentEntity, LightEntity, PresenceEntity,
};

#[derive(Default)]
pub struct EntitiesQuery;

#[Object]
impl EntitiesQuery {
    /// Every configured entity and its current state. Entity types the caller
    /// lacks a `graphql:<type>:read` scope for are silently omitted — this query
    /// never errors on missing permissions.
    async fn entities(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<Entity>> {
        let registry = ctx.data::<Arc<DeviceRegistry>>()?;
        let auth = ctx.data::<AuthContext>()?;

        let mut out = Vec::new();

        if auth.has(&required::GRAPHQL_LIGHT_READ) {
            out.extend(registry.lights().map(|(address, name)| {
                let id = registry.id_for_address(address).unwrap_or(address).to_owned();
                Entity::Light(LightEntity {
                    id,
                    name: name.clone(),
                    address: address.clone(),
                })
            }));
        }

        if auth.has(&required::GRAPHQL_DOOR_READ) {
            out.extend(registry.doors().map(|(address, settings)| {
                Entity::Door(DoorEntity {
                    id: settings.id.clone(),
                    name: settings.name.clone(),
                    address: address.clone(),
                })
            }));
        }

        if auth.has(&required::GRAPHQL_PRESENCE_READ) {
            out.extend(registry.presence_devices().map(|(address, settings)| {
                let id = registry.id_for_address(address).unwrap_or(address).to_owned();
                let name = if settings.name.is_empty() {
                    id.clone()
                } else {
                    settings.name.clone()
                };
                Entity::Presence(PresenceEntity {
                    id,
                    name,
                    address: address.clone(),
                })
            }));
        }

        if auth.has(&required::GRAPHQL_ENVIRONMENT_READ) {
            out.extend(registry.environment_devices().map(|(_address, settings)| {
                Entity::Environment(EnvironmentEntity {
                    id: settings.id.clone(),
                    name: settings.name.clone(),
                })
            }));
        }

        Ok(out)
    }
}
