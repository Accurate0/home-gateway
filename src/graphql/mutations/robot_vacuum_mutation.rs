use async_graphql::Object;
use serde_json::json;

use crate::auth::scope::required;
use crate::graphql::guard::ScopeGuard;
use crate::integrations::home_assistant::HomeAssistant;
use crate::integrations::mqtt::MqttClient;
use crate::settings::{RoborockSettings, ValetudoSettings};

#[derive(Clone, Copy)]
enum Action {
    Start,
    Stop,
    Dock,
}

enum Backend {
    Roborock {
        control_entity: String,
        start_service: String,
        stop_service: String,
        dock_service: String,
    },
    Valetudo {
        command_topic: String,
        start_payload: String,
        stop_payload: String,
        dock_payload: String,
    },
}

pub struct RobotVacuumMutation {
    backend: Backend,
}

impl RobotVacuumMutation {
    pub fn roborock(settings: &RoborockSettings) -> Self {
        Self {
            backend: Backend::Roborock {
                control_entity: settings.control_entity.clone(),
                start_service: settings.start_service.clone(),
                stop_service: settings.stop_service.clone(),
                dock_service: settings.dock_service.clone(),
            },
        }
    }

    pub fn valetudo(settings: &ValetudoSettings) -> Self {
        Self {
            backend: Backend::Valetudo {
                command_topic: settings.command_topic.clone(),
                start_payload: settings.start_payload.clone(),
                stop_payload: settings.stop_payload.clone(),
                dock_payload: settings.dock_payload.clone(),
            },
        }
    }

    async fn run(
        &self,
        ctx: &async_graphql::Context<'_>,
        action: Action,
    ) -> async_graphql::Result<bool> {
        match &self.backend {
            Backend::Roborock {
                control_entity,
                start_service,
                stop_service,
                dock_service,
            } => {
                let service = match action {
                    Action::Start => start_service,
                    Action::Stop => stop_service,
                    Action::Dock => dock_service,
                };

                let home_assistant = ctx.data::<Option<HomeAssistant>>()?;
                let Some(home_assistant) = home_assistant else {
                    return Err(async_graphql::Error::new(
                        "home assistant is not configured",
                    ));
                };

                let Some((domain, service)) = service.split_once('.') else {
                    return Err(async_graphql::Error::new(format!(
                        "invalid service `{service}`, expected `domain.service`"
                    )));
                };

                home_assistant
                    .call_service(domain, service, json!({ "entity_id": control_entity }))
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?;

                Ok(true)
            }
            Backend::Valetudo {
                command_topic,
                start_payload,
                stop_payload,
                dock_payload,
            } => {
                let payload = match action {
                    Action::Start => start_payload,
                    Action::Stop => stop_payload,
                    Action::Dock => dock_payload,
                };

                let mqtt = ctx.data::<MqttClient>()?;
                mqtt.send_event_raw(command_topic.clone(), payload)
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?;

                Ok(true)
            }
        }
    }
}

#[Object]
impl RobotVacuumMutation {
    #[graphql(guard = ScopeGuard(required::GRAPHQL_ROBOT_VACUUM_WRITE))]
    async fn start(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, Action::Start).await
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_ROBOT_VACUUM_WRITE))]
    async fn stop(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, Action::Stop).await
    }

    #[graphql(guard = ScopeGuard(required::GRAPHQL_ROBOT_VACUUM_WRITE))]
    async fn dock(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        self.run(ctx, Action::Dock).await
    }
}
