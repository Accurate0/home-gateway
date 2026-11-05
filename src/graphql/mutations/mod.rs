use async_graphql::MergedObject;

use crate::graphql::mutations::adhoc_mutation::{AdhocMutation, AdhocMutationV2};
use crate::graphql::mutations::entities_mutation::{EntitiesMutation, EntitiesMutationV2};
use crate::graphql::mutations::lua_mutation::{LuaMutation, LuaMutationV2};
use crate::graphql::mutations::push_mutation::{PushMutation, PushMutationV2};
use crate::graphql::mutations::workflows_mutation::{WorkflowsMutation, WorkflowsMutationV2};

pub mod adhoc_mutation;
pub mod air_purifier_mutation;
pub mod device_lock_mutation;
pub mod eink_display_mutation;
pub mod entities_mutation;
pub mod garage_door_mutation;
pub mod light_mutation;
pub mod lua_mutation;
pub mod media_player_mutation;
pub mod push_mutation;
pub mod robot_vacuum_mutation;
pub mod workflows_mutation;

#[derive(Default, MergedObject)]
#[graphql(name = "Mutation")]
pub struct MutationRoot(
    EntitiesMutation,
    WorkflowsMutation,
    AdhocMutation,
    PushMutation,
    LuaMutation,
);

#[derive(Default, MergedObject)]
#[graphql(name = "Mutation")]
pub struct MutationRootV2(
    EntitiesMutationV2,
    WorkflowsMutationV2,
    AdhocMutationV2,
    PushMutationV2,
    LuaMutationV2,
);
