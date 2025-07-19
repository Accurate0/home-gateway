use async_graphql::{EmptyMutation, EmptySubscription, MergedObject, Schema};
use queries::{
    energy_query::EnergyQuery, environments_query::EnvironmentsQuery, events_query::EventsQuery,
    solar_query::SolarQuery,
};

pub mod dataloader;
pub mod handler;
mod objects;
mod queries;

#[derive(Default, MergedObject)]
pub struct QueryRoot(EventsQuery, EnvironmentsQuery, SolarQuery, EnergyQuery);

pub type FinalSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;
