use serde_json::json;

use crate::device_registry::DeviceRegistry;
use crate::integrations::home_assistant::{HomeAssistant, HomeAssistantError};
use crate::integrations::mqtt::{MqttClient, MqttError, TopicVars};
use crate::settings::workflow::VacuumCommand;
use crate::settings::{RobotVacuumSettings, VacuumTarget};

#[derive(Debug, thiserror::Error)]
pub enum VacuumCommandError {
    #[error("home assistant is not configured")]
    HomeAssistantNotConfigured,
    #[error(transparent)]
    HomeAssistant(#[from] HomeAssistantError),
    #[error(transparent)]
    Mqtt(#[from] MqttError),
    #[error("no mqtt command topic: {0}")]
    CommandTopic(String),
}

pub async fn send(
    settings: &RobotVacuumSettings,
    command: VacuumCommand,
    home_assistant: Option<&HomeAssistant>,
    mqtt: &MqttClient,
    devices: &DeviceRegistry,
) -> Result<(), VacuumCommandError> {
    let instruction = settings.commands.get(command);

    match &settings.target {
        VacuumTarget::HomeAssistant { entity_id } => {
            let home_assistant =
                home_assistant.ok_or(VacuumCommandError::HomeAssistantNotConfigured)?;

            let Some((domain, service)) = instruction.split_once('.') else {
                return Err(HomeAssistantError::InvalidService(instruction.to_owned()).into());
            };

            home_assistant
                .call_service(domain, service, json!({ "entity_id": entity_id }))
                .await?;
        }
        VacuumTarget::Mqtt { address } => {
            let topic = devices
                .mqtt_command_topic(address, TopicVars::new())
                .await
                .map_err(VacuumCommandError::CommandTopic)?;

            mqtt.send_event_raw(topic, instruction, false).await?;
        }
    }

    Ok(())
}
