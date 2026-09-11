use async_graphql::Object;

use crate::actors::devices::robot_vacuum::command;
use crate::auth::scope::{Action as ScopeAction, Resource, Scope};
use crate::graphql::guard::ScopeGuard;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::mqtt::MqttClient;
use crate::settings::workflow::VacuumCommand;
use crate::settings::{RoborockSettings, ValetudoSettings};

enum Backend {
    Roborock(RoborockSettings),
    Valetudo(ValetudoSettings),
}

pub struct RobotVacuumMutation {
    backend: Backend,
}

impl RobotVacuumMutation {
    pub fn roborock(settings: &RoborockSettings) -> Self {
        Self {
            backend: Backend::Roborock(settings.clone()),
        }
    }

    pub fn valetudo(settings: &ValetudoSettings) -> Self {
        Self {
            backend: Backend::Valetudo(settings.clone()),
        }
    }

    async fn run(
        &self,
        ctx: &async_graphql::Context<'_>,
        vacuum_command: VacuumCommand,
    ) -> async_graphql::Result<bool> {
        match &self.backend {
            Backend::Roborock(settings) => {
                let home_assistant =
                    crate::graphql::require::<HomeAssistant>(ctx, "home assistant")?;

                command::roborock(home_assistant, settings, vacuum_command)
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?;
            }
            Backend::Valetudo(settings) => {
                let mqtt = crate::graphql::require::<MqttClient>(ctx, "mqtt")?;

                command::valetudo(mqtt, settings, vacuum_command)
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?;
            }
        }

        Ok(true)
    }
}

#[Object]
impl RobotVacuumMutation {
    #[graphql(guard = ScopeGuard(Scope::new(Resource::RobotVacuum, ScopeAction::Write)))]
    async fn start(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, VacuumCommand::Start).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::RobotVacuum, ScopeAction::Write)))]
    async fn stop(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, VacuumCommand::Stop).await
    }

    #[graphql(guard = ScopeGuard(Scope::new(Resource::RobotVacuum, ScopeAction::Write)))]
    async fn dock(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, VacuumCommand::Dock).await
    }
}
