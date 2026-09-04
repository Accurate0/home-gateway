use async_graphql::Object;

use crate::actors::eink_display::{EInkDisplayActor, EInkDisplayMessage};
use crate::auth::scope::{Action, Resource, Scope};
use crate::graphql::guard::ScopeGuard;

pub struct EinkDisplayMutation {
    pub address: String,
}

#[Object]
impl EinkDisplayMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::Epd, Action::Write)))]
    async fn take_screenshot(&self) -> async_graphql::Result<bool> {
        let Some(actor) = ractor::registry::where_is(EInkDisplayActor::NAME) else {
            return Err(async_graphql::Error::new("eink display actor unavailable"));
        };

        actor.send_message(EInkDisplayMessage::TakeScreenshot {
            device_id: Some(self.address.clone()),
        })?;

        Ok(true)
    }
}
