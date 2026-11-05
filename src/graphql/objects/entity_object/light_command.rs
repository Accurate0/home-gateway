use async_graphql::{Enum, SimpleObject};

use super::light_state::LightStateObject;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(name = "HomeLightCommandStatus")]
pub enum LightCommandStatusObject {
    Confirmed,
    Pending,
}

#[derive(SimpleObject)]
#[graphql(name = "HomeLightCommandResult")]
pub struct LightCommandResultObject {
    pub status: LightCommandStatusObject,
    pub state: LightStateObject,
}
