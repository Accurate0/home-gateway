use async_graphql::Object;

use crate::actors::devices::robot_vacuum::command;
use crate::auth::scope::{Action as ScopeAction, Resource, Scope};
use crate::device_registry::DeviceRegistry;
use crate::graphql::guard::ScopeGuard;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::mqtt::MqttClient;
use crate::settings::RobotVacuumSettings;
use crate::settings::workflow::VacuumCommand;

pub struct RobotVacuumMutation {
    settings: RobotVacuumSettings,
}

impl RobotVacuumMutation {
    pub fn new(settings: &RobotVacuumSettings) -> Self {
        Self {
            settings: settings.clone(),
        }
    }

    async fn run(
        &self,
        ctx: &async_graphql::Context<'_>,
        vacuum_command: VacuumCommand,
    ) -> async_graphql::Result<bool> {
        let mqtt = crate::graphql::require::<MqttClient>(ctx, "mqtt")?;
        let home_assistant = ctx
            .data::<crate::state::HandleRegistry>()?
            .get::<HomeAssistant>();
        let devices = ctx.data::<DeviceRegistry>()?;

        command::send(
            &self.settings,
            vacuum_command,
            home_assistant,
            mqtt,
            devices,
        )
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

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
