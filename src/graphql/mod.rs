use async_graphql::{MergedObject, Schema};
use queries::{
    adhoc_query::AdhocQuery, auth_query::AuthQuery, energy_query::EnergyQuery,
    entities_query::EntitiesQuery, home_assistant_query::HomeAssistantQuery,
    jellyfin_query::JellyfinQuery, solar_query::SolarQuery, weather_query::WeatherQuery,
};

use crate::graphql::mutations::MutationRoot;
use crate::graphql::queries::woolworths_query::WoolworthsQuery;
use crate::graphql::queries::workflows_query::WorkflowsQuery;
use crate::graphql::subscription::SubscriptionRoot;

pub mod dataloader;
pub mod guard;
pub mod handler;
pub mod mutations;
mod objects;
mod queries;
pub mod subscription;

#[derive(Default, MergedObject)]
pub struct QueryRoot(
    AuthQuery,
    HomeAssistantQuery,
    EntitiesQuery,
    SolarQuery,
    EnergyQuery,
    WeatherQuery,
    WoolworthsQuery,
    WorkflowsQuery,
    JellyfinQuery,
    AdhocQuery,
);

pub type FinalSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn sdl() -> String {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot,
    )
    .finish()
    .sdl()
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
}
