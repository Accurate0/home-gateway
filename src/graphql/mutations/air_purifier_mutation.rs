use async_graphql::Object;

use crate::actors::devices::air_purifier::command;
use crate::auth::scope::{Action as ScopeAction, Resource, Scope};
use crate::device_command::CommandOutcome;
use crate::graphql::guard::ScopeGuard;
use crate::repo::air_purifier::AirPurifierMode;
use crate::workflows::definition::AirPurifierCommand;

pub struct AirPurifierMutation {
    address: String,
}

impl AirPurifierMutation {
    pub fn new(address: String) -> Self {
        Self { address }
    }

    async fn run(
        &self,
        air_purifier_command: AirPurifierCommand,
    ) -> async_graphql::Result<CommandOutcome> {
        command::send(&self.address, air_purifier_command)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }
}

#[Object]
impl AirPurifierMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::AirPurifier, ScopeAction::Write)))]
    async fn turn_on(&self) -> async_graphql::Result<CommandOutcome> {
        self.run(AirPurifierCommand::TurnOn).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::AirPurifier, ScopeAction::Write)))]
    async fn turn_off(&self) -> async_graphql::Result<CommandOutcome> {
        self.run(AirPurifierCommand::TurnOff).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::AirPurifier, ScopeAction::Write)))]
    async fn set_mode(&self, mode: AirPurifierMode) -> async_graphql::Result<CommandOutcome> {
        self.run(AirPurifierCommand::SetMode { mode }).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::AirPurifier, ScopeAction::Write)))]
    async fn set_speed(&self, speed: u8) -> async_graphql::Result<CommandOutcome> {
        self.run(AirPurifierCommand::SetSpeed { speed }).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::AirPurifier, ScopeAction::Write)))]
    async fn set_display(&self, on: bool) -> async_graphql::Result<CommandOutcome> {
        self.run(AirPurifierCommand::SetDisplay { on }).await
    }
}
