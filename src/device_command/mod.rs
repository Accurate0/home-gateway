mod command_targets;
mod device_command_error;
mod outbound;

pub use command_targets::CommandTargets;
pub use device_command_error::DeviceCommandError;
pub use outbound::Outbound;

use serde_json::json;

use crate::decoding::DeviceRoleName;
use crate::device_registry::{DeviceRegistry, Transport};
use crate::integrations::esphome::EsphomeDomain;
use crate::integrations::home_assistant::HomeAssistantError;
use crate::integrations::mqtt::{MqttProtocol, TopicVars};

pub async fn send(
    targets: &CommandTargets<'_>,
    address: &str,
    role: DeviceRoleName,
    outbound: Outbound,
) -> Result<(), DeviceCommandError> {
    let device = targets
        .devices
        .device(address)
        .ok_or_else(|| DeviceCommandError::UnknownDevice(address.to_owned()))?;

    match device.transport {
        Transport::Mqtt => {
            let vars = topic_vars(targets.devices, address, role)?;

            let topic = targets
                .devices
                .mqtt_command_topic(address, vars)
                .await
                .map_err(DeviceCommandError::CommandTopic)?;

            match outbound {
                Outbound::Json(payload) => targets.mqtt.send_event(topic, payload).await?,
                Outbound::Text(payload) => {
                    targets.mqtt.send_event_raw(topic, &payload, false).await?
                }
            }
        }
        Transport::EsphomeNativeApi => {
            let Outbound::Json(payload) = outbound else {
                return Err(DeviceCommandError::ServiceForPayload(address.to_owned()));
            };

            if role != DeviceRoleName::Light {
                return Err(DeviceCommandError::Unsupported {
                    address: address.to_owned(),
                    transport: device.transport,
                    role,
                });
            }

            targets
                .esphome_native_api
                .ok_or(DeviceCommandError::EsphomeNativeApiNotConfigured)?
                .light(address, payload)
                .await?;
        }
        Transport::HomeAssistant => {
            let Outbound::Text(service) = outbound else {
                return Err(DeviceCommandError::PayloadForService(address.to_owned()));
            };

            let home_assistant = targets
                .home_assistant
                .ok_or(DeviceCommandError::HomeAssistantNotConfigured)?;

            let Some((domain, service)) = service.split_once('.') else {
                return Err(HomeAssistantError::InvalidService(service).into());
            };

            home_assistant
                .call_service(domain, service, json!({ "entity_id": address }))
                .await?;
        }
        Transport::EinkDisplayFirmware | Transport::Trmnl => {
            return Err(DeviceCommandError::Unsupported {
                address: address.to_owned(),
                transport: device.transport,
                role,
            });
        }
    }

    Ok(())
}

fn topic_vars(
    devices: &DeviceRegistry,
    address: &str,
    role: DeviceRoleName,
) -> Result<TopicVars, DeviceCommandError> {
    let protocol = devices
        .device(address)
        .and_then(|device| device.profile.as_ref()?.protocol);

    match (protocol, role) {
        (Some(MqttProtocol::Esphome), DeviceRoleName::Light) => {
            let object_id = devices.esphome_light(address).ok_or_else(|| {
                DeviceCommandError::CommandTopic(format!("{address} has no esphome light entity"))
            })?;

            Ok(TopicVars::from([
                ("domain".to_owned(), EsphomeDomain::Light.to_string()),
                ("object_id".to_owned(), object_id.to_owned()),
            ]))
        }
        (Some(MqttProtocol::Esphome), _) => Err(DeviceCommandError::Unsupported {
            address: address.to_owned(),
            transport: Transport::Mqtt,
            role,
        }),
        (Some(MqttProtocol::Zigbee | MqttProtocol::Valetudo) | None, _) => Ok(TopicVars::new()),
    }
}
