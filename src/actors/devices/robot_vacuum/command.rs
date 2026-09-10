use serde_json::json;

use crate::integrations::home_assistant::{HomeAssistant, HomeAssistantError};
use crate::integrations::mqtt::{MqttClient, MqttError};
use crate::settings::vacuum_command::VacuumCommand;
use crate::settings::{RoborockSettings, ValetudoSettings};

pub async fn roborock(
    home_assistant: &HomeAssistant,
    settings: &RoborockSettings,
    command: VacuumCommand,
) -> Result<(), HomeAssistantError> {
    let service = match command {
        VacuumCommand::Start => &settings.start_service,
        VacuumCommand::Stop => &settings.stop_service,
        VacuumCommand::Dock => &settings.dock_service,
    };

    let Some((domain, service)) = service.split_once('.') else {
        return Err(HomeAssistantError::InvalidService(service.clone()));
    };

    home_assistant
        .call_service(
            domain,
            service,
            json!({ "entity_id": settings.control_entity }),
        )
        .await
}

pub async fn valetudo(
    mqtt: &MqttClient,
    settings: &ValetudoSettings,
    command: VacuumCommand,
) -> Result<(), MqttError> {
    let payload = match command {
        VacuumCommand::Start => &settings.start_payload,
        VacuumCommand::Stop => &settings.stop_payload,
        VacuumCommand::Dock => &settings.dock_payload,
    };

    mqtt.send_event_raw(settings.command_topic.clone(), payload, false)
        .await
}
