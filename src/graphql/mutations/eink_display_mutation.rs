use async_graphql::Object;

use crate::actors::eink_display::{EInkDisplayActor, EInkDisplayMessage};
use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;

pub struct EinkDisplayMutation {
    pub address: String,
}

#[Object]
impl EinkDisplayMutation {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_EPD_WRITE))]
    async fn take_screenshot(&self) -> async_graphql::Result<bool> {
        let Some(actor) = ractor::registry::where_is(EInkDisplayActor::NAME.to_string()) else {
            return Err(async_graphql::Error::new("eink display actor unavailable"));
        };

        actor.send_message(EInkDisplayMessage::TakeScreenshot {
            device_id: Some(self.address.clone()),
        })?;

        Ok(true)
    }
}
