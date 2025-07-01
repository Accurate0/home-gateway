use async_graphql::{EmptyMutation, EmptySubscription, MergedObject, Schema};

pub mod handler;

#[derive(Default, MergedObject)]
pub struct QueryRoot();

pub type FinalSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;
