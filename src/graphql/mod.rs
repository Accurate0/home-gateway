use async_graphql::{MergedObject, Schema};
use queries::{
    auth_query::AuthQuery, energy_query::EnergyQuery, entities_query::EntitiesQuery,
    environments_query::EnvironmentsQuery, events_query::EventsQuery, solar_query::SolarQuery,
    weather_query::WeatherQuery,
};

use crate::graphql::mutations::MutationRoot;
use crate::graphql::queries::woolworths_query::WoolworthsQuery;
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
    EventsQuery,
    EnvironmentsQuery,
    EntitiesQuery,
    SolarQuery,
    EnergyQuery,
    WeatherQuery,
    WoolworthsQuery,
);

pub type FinalSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
