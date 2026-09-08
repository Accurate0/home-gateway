use async_graphql::{Enum, SimpleObject};

use super::light_state::LightStateObject;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(name = "LightCommandStatus")]
pub enum LightCommandStatusObject {
    Confirmed,
    Pending,
}

#[derive(SimpleObject)]
#[graphql(name = "LightCommandResult")]
pub struct LightCommandResultObject {
    pub status: LightCommandStatusObject,
    pub state: LightStateObject,
}
