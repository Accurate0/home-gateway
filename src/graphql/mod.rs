use async_graphql::{MergedObject, SDLExportOptions, Schema, SchemaBuilder};
use queries::{
    adhoc_query::{AdhocQuery, AdhocQueryV2},
    auth_query::{AuthQuery, AuthQueryV2},
    energy_query::{EnergyQuery, EnergyQueryV2},
    entities_query::{EntitiesQuery, EntitiesQueryV2},
    home_assistant_query::{HomeAssistantQuery, HomeAssistantQueryV2},
    jellyfin_query::{JellyfinQuery, JellyfinQueryV2},
    solar_query::{SolarQuery, SolarQueryV2},
    weather_query::{WeatherQuery, WeatherQueryV2},
};

use crate::graphql::mutations::{MutationRoot, MutationRootV2};
use crate::graphql::queries::fuelwatch_query::{FuelWatchQuery, FuelWatchQueryV2};
use crate::graphql::queries::lua_query::{LuaQuery, LuaQueryV2};
use crate::graphql::queries::push_query::{PushQuery, PushQueryV2};
use crate::graphql::queries::transperth_query::{TransperthQuery, TransperthQueryV2};
use crate::graphql::queries::unraid_query::{UnraidQuery, UnraidQueryV2};
use crate::graphql::queries::woolworths_query::{WoolworthsQuery, WoolworthsQueryV2};
use crate::graphql::queries::workflows_query::{WorkflowsQuery, WorkflowsQueryV2};
use crate::graphql::subscription::{SubscriptionRoot, SubscriptionRootV2};

pub mod dataloader;
pub mod guard;
pub mod handler;
pub mod mutations;
mod objects;
mod queries;
pub mod subscription;

pub use objects::lockable::LockableEntity;

#[derive(Default, MergedObject)]
#[graphql(name = "Query")]
pub struct QueryRoot(
    AuthQuery,
    HomeAssistantQuery,
    EntitiesQuery,
    SolarQuery,
    EnergyQuery,
    WeatherQuery,
    WoolworthsQuery,
    TransperthQuery,
    FuelWatchQuery,
    UnraidQuery,
    WorkflowsQuery,
    JellyfinQuery,
    AdhocQuery,
    PushQuery,
    LuaQuery,
);

/// The supergraph's query root: every v1 root field prefixed with `home`.
#[derive(Default, MergedObject)]
#[graphql(name = "Query")]
pub struct QueryRootV2(
    AuthQueryV2,
    HomeAssistantQueryV2,
    EntitiesQueryV2,
    SolarQueryV2,
    EnergyQueryV2,
    WeatherQueryV2,
    WoolworthsQueryV2,
    TransperthQueryV2,
    FuelWatchQueryV2,
    UnraidQueryV2,
    WorkflowsQueryV2,
    JellyfinQueryV2,
    AdhocQueryV2,
    PushQueryV2,
    LuaQueryV2,
);

pub type FinalSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
pub type FinalSchemaV2 = Schema<QueryRootV2, MutationRootV2, SubscriptionRootV2>;

pub fn require<'a, T: std::any::Any + Send + Sync>(
    ctx: &'a async_graphql::Context<'_>,
    name: &str,
) -> async_graphql::Result<&'a T> {
    ctx.data::<crate::state::HandleRegistry>()?
        .get::<T>()
        .ok_or_else(|| async_graphql::Error::new(format!("{name} is not configured")))
}

pub fn builder() -> SchemaBuilder<QueryRoot, MutationRoot, SubscriptionRoot> {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot,
    )
    .register_output_type::<LockableEntity>()
    .enable_federation()
    .enable_subscription_in_federation()
}

pub fn builder_v2() -> SchemaBuilder<QueryRootV2, MutationRootV2, SubscriptionRootV2> {
    Schema::build(
        QueryRootV2::default(),
        MutationRootV2::default(),
        SubscriptionRootV2,
    )
    .register_output_type::<LockableEntity>()
    .enable_federation()
    .enable_subscription_in_federation()
}

pub fn sdl() -> String {
    builder().finish().sdl()
}

pub fn subgraph_sdl() -> String {
    builder_v2()
        .finish()
        .sdl_with_options(SDLExportOptions::new().federation())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_reviewed() {
        let sdl = sdl();

        insta::assert_snapshot!("schema", &sdl);

        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("schema.graphql");
        let current = std::fs::read_to_string(&path).unwrap_or_default();

        if current != sdl {
            std::fs::write(&path, &sdl).expect("write schema.graphql");
        }
    }

    #[test]
    fn v2_prefixes_every_v1_root_field() {
        use async_graphql::parser::types::{TypeKind, TypeSystemDefinition};

        let roots = |sdl: String| {
            let doc = async_graphql::parser::parse_schema(sdl).expect("parse sdl");
            let mut fields = Vec::new();
            for definition in doc.definitions {
                let TypeSystemDefinition::Type(ty) = definition else {
                    continue;
                };
                let name = ty.node.name.node.to_string();
                if !matches!(name.as_str(), "Query" | "Mutation" | "Subscription") {
                    continue;
                }
                if let TypeKind::Object(object) = ty.node.kind {
                    for field in object.fields {
                        let field = field.node.name.node.to_string();
                        if !field.starts_with('_') {
                            fields.push(format!("{name}.{field}"));
                        }
                    }
                }
            }
            fields.sort();
            fields
        };

        let mut prefixed: Vec<String> = roots(sdl())
            .into_iter()
            .map(|path| {
                let (ty, field) = path.split_once('.').unwrap();
                let mut chars = field.chars();
                let first = chars.next().unwrap().to_ascii_uppercase();
                format!("{ty}.home{first}{}", chars.as_str())
            })
            .collect();
        prefixed.sort();

        assert_eq!(roots(builder_v2().finish().sdl()), prefixed);
    }

    #[tokio::test]
    async fn v2_keeps_v1_guards() {
        let response = builder_v2()
            .finish()
            .execute("{ homeSolar { __typename } }")
            .await;

        assert!(
            response.errors[0].message.contains("AuthContext"),
            "{:?}",
            response.errors
        );
    }

    #[test]
    fn subgraph_is_reviewed() {
        let sdl = subgraph_sdl();

        insta::assert_snapshot!("subgraph", &sdl);

        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("subgraph.graphql");
        let current = std::fs::read_to_string(&path).unwrap_or_default();

        if current != sdl {
            std::fs::write(&path, &sdl).expect("write subgraph.graphql");
        }
    }
}
