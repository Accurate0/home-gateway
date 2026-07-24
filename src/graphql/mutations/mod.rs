use async_graphql::MergedObject;

use crate::graphql::mutations::entities_mutation::EntitiesMutation;
use crate::graphql::mutations::workflows_mutation::WorkflowsMutation;

pub mod entities_mutation;
pub mod light_mutation;
pub mod roborock_mutation;
pub mod workflows_mutation;

#[derive(Default, MergedObject)]
pub struct MutationRoot(EntitiesMutation, WorkflowsMutation);
