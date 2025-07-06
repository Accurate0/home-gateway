use async_graphql::{EmptyMutation, EmptySubscription, MergedObject, Schema};
use queries::events_query::EventsQuery;

pub mod dataloader;
pub mod handler;
mod objects;
mod queries;

#[derive(Default, MergedObject)]
pub struct QueryRoot(EventsQuery);

pub type FinalSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;
