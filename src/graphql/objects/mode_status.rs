use async_graphql::Object;

use crate::graphql::objects::mode_object::ModeObject;
use crate::mode::Mode;

pub struct ModeStatus {
    pub active: Mode,
}

#[Object]
impl ModeStatus {
    async fn active(&self) -> Mode {
        self.active
    }

    async fn node(&self, mode: Option<Mode>) -> ModeObject {
        ModeObject::new(mode.unwrap_or(self.active))
    }
}
