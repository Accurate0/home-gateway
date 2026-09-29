use async_graphql::Object;

use crate::actors::devices::garage_door::{CommandOutcome, command};
use crate::auth::scope::{Action as ScopeAction, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::settings::workflow::GarageDoorCommand;

pub struct GarageDoorMutation {
    address: String,
}

impl GarageDoorMutation {
    pub fn new(address: String) -> Self {
        Self { address }
    }

    async fn run(
        &self,
        garage_door_command: GarageDoorCommand,
    ) -> async_graphql::Result<CommandOutcome> {
        command::send(&self.address, garage_door_command)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }
}

#[Object]
impl GarageDoorMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::GarageDoor, ScopeAction::Write)))]
    async fn open(&self) -> async_graphql::Result<CommandOutcome> {
        self.run(GarageDoorCommand::Open).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::GarageDoor, ScopeAction::Write)))]
    async fn close(&self) -> async_graphql::Result<CommandOutcome> {
        self.run(GarageDoorCommand::Close).await
    }
}
