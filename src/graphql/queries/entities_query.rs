use async_graphql::Object;

use crate::auth::context::AuthContext;
use crate::auth::scope::required;
use crate::device_registry::DeviceRegistry;
use crate::graphql::objects::entity_object::{
    DoorEntity, EinkDisplayEntity, EinkDisplayKind, Entity, EntitySection, EnvironmentEntity,
    LightEntity, PresenceEntity, RoborockEntity, ValetudoEntity,
};

#[derive(Default)]
pub struct EntitiesQuery;

#[Object]
impl EntitiesQuery {
    /// Every configured entity and its current state. Entity types the caller
    /// lacks a `graphql:<type>:read` scope for are silently omitted — this query
    /// never errors on missing permissions.
    /// The dashboard sections, in display order, with their titles. Categorising
    /// a device kind is a backend concern — the frontend renders whatever order
    /// and titles this returns.
    async fn entity_sections(&self) -> Vec<EntitySection> {
        EntitySection::ordered()
    }

    async fn entities(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<Entity>> {
        let registry = ctx.data::<DeviceRegistry>()?;
        let auth = ctx.data::<AuthContext>()?;

        let mut out = Vec::new();

        if auth.has(&required::GRAPHQL_LIGHT_READ) {
            out.extend(registry.lights().map(|(address, name)| {
                let id = registry
                    .id_for_address(address)
                    .unwrap_or(address)
                    .to_owned();
                Entity::Light(LightEntity {
                    id,
                    name: name.clone(),
                    address: address.clone(),
                    capabilities: registry.capabilities(address).to_vec(),
                    room: registry.room(address).map(str::to_owned),
                })
            }));
        }

        if auth.has(&required::GRAPHQL_DOOR_READ) {
            out.extend(registry.doors().map(|(address, settings)| {
                Entity::Door(DoorEntity {
                    id: settings.id.clone(),
                    name: settings.name.clone(),
                    address: address.clone(),
                    capabilities: registry.capabilities(address).to_vec(),
                    room: registry.room(address).map(str::to_owned),
                })
            }));
        }

        if auth.has(&required::GRAPHQL_PRESENCE_READ) {
            out.extend(registry.presence_devices().map(|(address, settings)| {
                let id = registry
                    .id_for_address(address)
                    .unwrap_or(address)
                    .to_owned();
                let name = if settings.name.is_empty() {
                    id.clone()
                } else {
                    settings.name.clone()
                };
                Entity::Presence(PresenceEntity {
                    id,
                    name,
                    address: address.clone(),
                    capabilities: registry.capabilities(address).to_vec(),
                    room: registry.room(address).map(str::to_owned),
                })
            }));
        }

        if auth.has(&required::GRAPHQL_ENVIRONMENT_READ) {
            out.extend(registry.environment_devices().map(|(address, settings)| {
                Entity::Environment(EnvironmentEntity {
                    id: settings.id.clone(),
                    name: settings.name.clone(),
                    address: address.clone(),
                    capabilities: registry.capabilities(address).to_vec(),
                    room: registry.room(address).map(str::to_owned),
                })
            }));
        }

        if auth.has(&required::GRAPHQL_EPD_READ) {
            out.extend(registry.eink_displays().iter().map(|(address, settings)| {
                let id = registry
                    .id_for_address(address)
                    .unwrap_or(address)
                    .to_owned();
                Entity::EinkDisplay(EinkDisplayEntity {
                    id,
                    name: settings.name.clone(),
                    address: address.clone(),
                    kind: EinkDisplayKind::EinkDisplayFirmware,
                    capabilities: registry.capabilities(address).to_vec(),
                    room: registry.room(address).map(str::to_owned),
                })
            }));

            out.extend(registry.trmnl_devices().iter().map(|(address, settings)| {
                Entity::EinkDisplay(EinkDisplayEntity {
                    id: settings.id.clone(),
                    name: settings.name.clone(),
                    address: settings.id.clone(),
                    kind: EinkDisplayKind::Trmnl,
                    capabilities: registry.capabilities(address).to_vec(),
                    room: registry.room(address).map(str::to_owned),
                })
            }));
        }

        if auth.has(&required::GRAPHQL_ROBOROCK_READ) {
            out.extend(registry.roborocks().map(|(address, settings)| {
                let id = registry
                    .id_for_address(address)
                    .unwrap_or(address)
                    .to_owned();
                Entity::Roborock(RoborockEntity {
                    id,
                    name: settings.name.clone(),
                    room: registry.room(address).map(str::to_owned),
                    capabilities: registry.capabilities(address).to_vec(),
                    status_entity: settings.status_entity.clone(),
                    battery_entity: settings.battery_entity.clone(),
                    room_entity: settings.room_entity.clone(),
                })
            }));
        }

        if auth.has(&required::GRAPHQL_VALETUDO_READ) {
            out.extend(registry.valetudos().map(|(address, settings)| {
                let id = registry
                    .id_for_address(address)
                    .unwrap_or(address)
                    .to_owned();
                Entity::Valetudo(ValetudoEntity {
                    id,
                    name: settings.name.clone(),
                    room: registry.room(address).map(str::to_owned),
                    capabilities: registry.capabilities(address).to_vec(),
                    identifier: address.clone(),
                })
            }));
        }

        Ok(out)
    }
}
