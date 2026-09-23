mod media_command;
mod media_control_error;

pub use media_command::MediaCommand;
pub use media_control_error::MediaControlError;

use serde_json::{Value, json};

use crate::device_command::CommandTargets;
use crate::device_registry::Transport;

pub async fn send(
    targets: &CommandTargets<'_>,
    address: &str,
    command: MediaCommand,
) -> Result<(), MediaControlError> {
    let device = targets
        .devices
        .device(address)
        .ok_or_else(|| MediaControlError::UnknownDevice(address.to_owned()))?;

    match device.transport {
        Transport::HomeAssistant => {
            let home_assistant = targets
                .home_assistant
                .ok_or(MediaControlError::HomeAssistantNotConfigured)?;

            let (service, extra) = home_assistant_service(&command);

            let mut data = json!({ "entity_id": address });

            if let (Some(target), Some(extra)) = (data.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    target.insert(key.clone(), value.clone());
                }
            }

            home_assistant
                .call_service("media_player", service, data)
                .await?;
        }
        Transport::EsphomeNativeApi => {
            targets
                .esphome_native_api
                .ok_or(MediaControlError::EsphomeNativeApiNotConfigured)?
                .media_player(address, command)
                .await?;
        }
        transport => {
            return Err(MediaControlError::Unsupported {
                address: address.to_owned(),
                transport,
            });
        }
    }

    Ok(())
}

fn home_assistant_service(command: &MediaCommand) -> (&'static str, Value) {
    match command {
        MediaCommand::Play => ("media_play", json!({})),
        MediaCommand::Pause => ("media_pause", json!({})),
        MediaCommand::PlayPause => ("media_play_pause", json!({})),
        MediaCommand::Stop => ("media_stop", json!({})),
        MediaCommand::Next => ("media_next_track", json!({})),
        MediaCommand::Previous => ("media_previous_track", json!({})),
        MediaCommand::Volume(volume) => ("volume_set", json!({ "volume_level": volume })),
        MediaCommand::Mute(muted) => ("volume_mute", json!({ "is_volume_muted": muted })),
        MediaCommand::TurnOff => ("turn_off", json!({})),
        MediaCommand::PlayMedia { url, announcement } => (
            "play_media",
            json!({
                "media_content_id": url,
                "media_content_type": "music",
                "announce": announcement,
            }),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_maps_onto_a_home_assistant_service() {
        assert_eq!(home_assistant_service(&MediaCommand::Play).0, "media_play");
        assert_eq!(
            home_assistant_service(&MediaCommand::Next).0,
            "media_next_track"
        );

        let (service, data) = home_assistant_service(&MediaCommand::Volume(0.4));
        assert_eq!(service, "volume_set");
        assert_eq!(data["volume_level"], json!(0.4));

        let (service, data) = home_assistant_service(&MediaCommand::PlayMedia {
            url: "http://media/track.flac".to_owned(),
            announcement: true,
        });
        assert_eq!(service, "play_media");
        assert_eq!(data["media_content_id"], json!("http://media/track.flac"));
        assert_eq!(data["announce"], json!(true));
    }
}
