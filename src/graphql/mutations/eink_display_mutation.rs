use async_graphql::Object;

use crate::actors::eink_display::{EInkDisplayActor, EInkDisplayMessage};
use crate::actors::system::rpc;
use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;

pub struct EinkDisplayMutation {
    pub address: String,
}

#[Object]
impl EinkDisplayMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Epd, Action::Write)))]
    async fn take_screenshot(&self) -> async_graphql::Result<bool> {
        rpc::cast(
            EInkDisplayActor::NAME,
            EInkDisplayMessage::TakeScreenshot {
                device_id: Some(self.address.clone()),
            },
        )?;

        Ok(true)
    }
}
