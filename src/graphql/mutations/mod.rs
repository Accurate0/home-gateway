use async_graphql::MergedObject;

use crate::graphql::mutations::entities_mutation::EntitiesMutation;

pub mod entities_mutation;
pub mod light_mutation;

#[derive(Default, MergedObject)]
pub struct MutationRoot(EntitiesMutation);
