use std::collections::HashMap;

use prost::Message as _;
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc;

use crate::media_control::MediaCommand;
use crate::settings::EsphomeSettings;

use super::command::Command;
use super::entity_domain::EntityDomain;
use super::error::EsphomeNativeApiError;
use super::frame;
use super::proto;
use super::state_update::StateUpdate;

const CLIENT_INFO: &str = "home-gateway";
const API_VERSION_MAJOR: u32 = 1;
const API_VERSION_MINOR: u32 = 10;

const HELLO_REQUEST: u16 = 1;
const HELLO_RESPONSE: u16 = 2;
const DISCONNECT_REQUEST: u16 = 5;
const DISCONNECT_RESPONSE: u16 = 6;
const PING_REQUEST: u16 = 7;
const PING_RESPONSE: u16 = 8;
const LIST_ENTITIES_REQUEST: u16 = 11;
const LIST_ENTITIES_BINARY_SENSOR: u16 = 12;
const LIST_ENTITIES_FAN: u16 = 14;
const LIST_ENTITIES_LIGHT: u16 = 15;
const LIST_ENTITIES_SENSOR: u16 = 16;
const LIST_ENTITIES_SWITCH: u16 = 17;
const LIST_ENTITIES_TEXT_SENSOR: u16 = 18;
const LIST_ENTITIES_DONE: u16 = 19;
const SUBSCRIBE_STATES_REQUEST: u16 = 20;
const BINARY_SENSOR_STATE: u16 = 21;
const FAN_STATE: u16 = 23;
const LIGHT_STATE: u16 = 24;
const SENSOR_STATE: u16 = 25;
const SWITCH_STATE: u16 = 26;
const TEXT_SENSOR_STATE: u16 = 27;
const FAN_COMMAND_REQUEST: u16 = 31;
const LIGHT_COMMAND_REQUEST: u16 = 32;
const SWITCH_COMMAND_REQUEST: u16 = 33;
const LIST_ENTITIES_NUMBER: u16 = 49;
const NUMBER_STATE: u16 = 50;
const LIST_ENTITIES_SELECT: u16 = 52;
const SELECT_STATE: u16 = 53;
const LIST_ENTITIES_MEDIA_PLAYER: u16 = 63;
const MEDIA_PLAYER_STATE: u16 = 64;
const MEDIA_PLAYER_COMMAND_REQUEST: u16 = 65;

struct Entity {
    domain: EntityDomain,
    object_id: String,
}

pub struct Session {
    address: String,
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
    cipher: snow::TransportState,
    entities: HashMap<u32, Entity>,
    light: Option<u32>,
    fan: Option<u32>,
    media_player: Option<u32>,
}

impl Session {
    pub async fn connect(
        address: &str,
        settings: &EsphomeSettings,
        key: &str,
    ) -> Result<Self, EsphomeNativeApiError> {
        let target = with_port(address, settings.port);

        tracing::debug!("connecting to esphome node {target}");

        let mut stream = TcpStream::connect(&target).await?;
        stream.set_nodelay(true)?;

        let cipher = frame::handshake(&mut stream, key).await?;
        let (reader, writer) = stream.into_split();

        let mut session = Session {
            address: address.to_owned(),
            reader,
            writer,
            cipher,
            entities: HashMap::new(),
            light: None,
            fan: None,
            media_player: None,
        };

        session
            .send(
                HELLO_REQUEST,
                &proto::HelloRequest {
                    client_info: CLIENT_INFO.to_owned(),
                    api_version_major: API_VERSION_MAJOR,
                    api_version_minor: API_VERSION_MINOR,
                },
            )
            .await?;

        session.list_entities().await?;

        session
            .send(SUBSCRIBE_STATES_REQUEST, &proto::SubscribeStatesRequest {})
            .await?;

        Ok(session)
    }

    async fn send<M: Message>(
        &mut self,
        message_type: u16,
        message: &M,
    ) -> Result<(), EsphomeNativeApiError> {
        let frame = frame::encrypt(&mut self.cipher, message_type, &message.encode_to_vec())?;

        frame::write_frame(&mut self.writer, &frame).await
    }

    async fn receive(&mut self) -> Result<(u16, Vec<u8>), EsphomeNativeApiError> {
        let frame = frame::read_frame(&mut self.reader).await?;

        frame::decrypt(&mut self.cipher, &frame)
    }

    async fn list_entities(&mut self) -> Result<(), EsphomeNativeApiError> {
        self.send(LIST_ENTITIES_REQUEST, &proto::ListEntitiesRequest {})
            .await?;

        loop {
            let (message_type, payload) = self.receive().await?;

            match message_type {
                HELLO_RESPONSE => {
                    let hello = proto::HelloResponse::decode(payload.as_slice())?;

                    tracing::info!(
                        "esphome node {} is {} running {}",
                        self.address,
                        hello.name,
                        hello.server_info
                    );
                }
                LIST_ENTITIES_SENSOR => {
                    let entity = proto::ListEntitiesSensorResponse::decode(payload.as_slice())?;
                    self.register(entity.key, EntityDomain::Sensor, entity.object_id);
                }
                LIST_ENTITIES_BINARY_SENSOR => {
                    let entity =
                        proto::ListEntitiesBinarySensorResponse::decode(payload.as_slice())?;
                    self.register(entity.key, EntityDomain::BinarySensor, entity.object_id);
                }
                LIST_ENTITIES_TEXT_SENSOR => {
                    let entity = proto::ListEntitiesTextSensorResponse::decode(payload.as_slice())?;
                    self.register(entity.key, EntityDomain::TextSensor, entity.object_id);
                }
                LIST_ENTITIES_LIGHT => {
                    let entity = proto::ListEntitiesLightResponse::decode(payload.as_slice())?;
                    self.light.get_or_insert(entity.key);
                    self.register(entity.key, EntityDomain::Light, entity.object_id);
                }
                LIST_ENTITIES_FAN => {
                    let entity = proto::ListEntitiesFanResponse::decode(payload.as_slice())?;
                    self.fan.get_or_insert(entity.key);
                    self.register(entity.key, EntityDomain::Fan, entity.object_id);
                }
                LIST_ENTITIES_SWITCH => {
                    let entity = proto::ListEntitiesSwitchResponse::decode(payload.as_slice())?;
                    self.register(entity.key, EntityDomain::Switch, entity.object_id);
                }
                LIST_ENTITIES_NUMBER => {
                    let entity = proto::ListEntitiesNumberResponse::decode(payload.as_slice())?;
                    self.register(entity.key, EntityDomain::Number, entity.object_id);
                }
                LIST_ENTITIES_SELECT => {
                    let entity = proto::ListEntitiesSelectResponse::decode(payload.as_slice())?;
                    self.register(entity.key, EntityDomain::Select, entity.object_id);
                }
                LIST_ENTITIES_MEDIA_PLAYER => {
                    let entity =
                        proto::ListEntitiesMediaPlayerResponse::decode(payload.as_slice())?;
                    self.media_player.get_or_insert(entity.key);
                    self.register(entity.key, EntityDomain::MediaPlayer, entity.object_id);
                }
                LIST_ENTITIES_DONE => {
                    tracing::info!(
                        "esphome node {} listed {} entities the gateway can read",
                        self.address,
                        self.entities.len()
                    );

                    return Ok(());
                }
                PING_REQUEST => self.send(PING_RESPONSE, &proto::PingResponse {}).await?,
                DISCONNECT_REQUEST => return Err(self.disconnect_error(&payload)),
                _ => {}
            }
        }
    }

    fn register(&mut self, key: u32, domain: EntityDomain, object_id: String) {
        self.entities.insert(key, Entity { domain, object_id });
    }

    fn entity_key(&self, domain: EntityDomain, object_id: &str) -> Option<u32> {
        self.entities
            .iter()
            .find(|(_, entity)| entity.domain == domain && entity.object_id == object_id)
            .map(|(key, _)| *key)
    }

    fn disconnect_error(&self, payload: &[u8]) -> EsphomeNativeApiError {
        let reason = proto::DisconnectRequest::decode(payload)
            .ok()
            .and_then(|request| proto::DisconnectReason::try_from(request.reason).ok())
            .unwrap_or(proto::DisconnectReason::Unspecified);

        EsphomeNativeApiError::Disconnected(reason.as_str_name().to_owned())
    }

    pub async fn run(
        &mut self,
        commands: &mut mpsc::Receiver<Command>,
        settings: &EsphomeSettings,
        mut on_state: impl FnMut(StateUpdate),
    ) -> Result<(), EsphomeNativeApiError> {
        let silence_timeout = settings.silence_timeout();

        let mut keep_alive = tokio::time::interval(settings.keep_alive());
        keep_alive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        keep_alive.tick().await;

        loop {
            let frame = tokio::select! {
                _ = keep_alive.tick() => {
                    self.send(PING_REQUEST, &proto::PingRequest {}).await?;
                    continue;
                }
                command = commands.recv() => {
                    match command {
                        Some(command) => self.command(command).await?,
                        None => return Ok(()),
                    }

                    continue;
                }
                frame = tokio::time::timeout(
                    silence_timeout,
                    frame::read_frame(&mut self.reader),
                ) => frame,
            };

            let frame = match frame {
                Ok(frame) => frame?,
                Err(_) => return Err(EsphomeNativeApiError::Silent(silence_timeout)),
            };

            let (message_type, payload) = frame::decrypt(&mut self.cipher, &frame)?;

            match message_type {
                PING_REQUEST => self.send(PING_RESPONSE, &proto::PingResponse {}).await?,
                PING_RESPONSE | HELLO_RESPONSE | DISCONNECT_RESPONSE => {}
                DISCONNECT_REQUEST => {
                    self.send(DISCONNECT_RESPONSE, &proto::DisconnectResponse {})
                        .await?;

                    return Err(self.disconnect_error(&payload));
                }
                _ => {
                    if let Some(update) = self.state(message_type, &payload)? {
                        on_state(update);
                    }
                }
            }
        }
    }

    fn state(
        &self,
        message_type: u16,
        payload: &[u8],
    ) -> Result<Option<StateUpdate>, EsphomeNativeApiError> {
        let (key, value) = match message_type {
            SENSOR_STATE => {
                let state = proto::SensorStateResponse::decode(payload)?;

                (
                    state.key,
                    (!state.missing_state).then(|| json!(f64::from(state.state))),
                )
            }
            BINARY_SENSOR_STATE => {
                let state = proto::BinarySensorStateResponse::decode(payload)?;

                (
                    state.key,
                    (!state.missing_state).then(|| json!(state.state)),
                )
            }
            TEXT_SENSOR_STATE => {
                let state = proto::TextSensorStateResponse::decode(payload)?;

                (
                    state.key,
                    (!state.missing_state).then(|| json!(state.state)),
                )
            }
            LIGHT_STATE => {
                let state = proto::LightStateResponse::decode(payload)?;

                (state.key, Some(light_state(&state)))
            }
            MEDIA_PLAYER_STATE => {
                let state = proto::MediaPlayerStateResponse::decode(payload)?;

                (state.key, Some(media_player_state(&state)))
            }
            FAN_STATE => {
                let state = proto::FanStateResponse::decode(payload)?;

                (state.key, Some(fan_state(&state)))
            }
            SWITCH_STATE => {
                let state = proto::SwitchStateResponse::decode(payload)?;

                (state.key, Some(json!(state.state)))
            }
            NUMBER_STATE => {
                let state = proto::NumberStateResponse::decode(payload)?;

                (
                    state.key,
                    (!state.missing_state).then(|| json!(f64::from(state.state))),
                )
            }
            SELECT_STATE => {
                let state = proto::SelectStateResponse::decode(payload)?;

                (
                    state.key,
                    (!state.missing_state).then(|| json!(state.state)),
                )
            }
            _ => return Ok(None),
        };

        let Some(entity) = self.entities.get(&key) else {
            return Ok(None);
        };

        let Some(payload) = value else {
            return Ok(None);
        };

        Ok(Some(StateUpdate {
            address: self.address.clone(),
            domain: entity.domain,
            object_id: entity.object_id.clone(),
            payload,
        }))
    }

    async fn command(&mut self, command: Command) -> Result<(), EsphomeNativeApiError> {
        match command {
            Command::Light(payload) => {
                let key = self.light.ok_or_else(|| EsphomeNativeApiError::NoEntity {
                    address: self.address.clone(),
                    domain: EntityDomain::Light.to_string(),
                })?;

                self.send(LIGHT_COMMAND_REQUEST, &light_command(key, &payload))
                    .await
            }
            Command::Fan(payload) => {
                if FAN_FIELDS.iter().any(|field| payload.get(field).is_some()) {
                    let key = self.fan.ok_or_else(|| EsphomeNativeApiError::NoEntity {
                        address: self.address.clone(),
                        domain: EntityDomain::Fan.to_string(),
                    })?;

                    self.send(FAN_COMMAND_REQUEST, &fan_command(key, &payload))
                        .await?;
                }

                for (object_id, state) in switch_commands(&payload) {
                    let key = self
                        .entity_key(EntityDomain::Switch, &object_id)
                        .ok_or_else(|| EsphomeNativeApiError::NoEntity {
                            address: self.address.clone(),
                            domain: format!("{} {object_id}", EntityDomain::Switch),
                        })?;

                    let request = proto::SwitchCommandRequest {
                        key,
                        state,
                        ..Default::default()
                    };

                    self.send(SWITCH_COMMAND_REQUEST, &request).await?;
                }

                Ok(())
            }
            Command::Media(command) => {
                let key = self
                    .media_player
                    .ok_or_else(|| EsphomeNativeApiError::NoEntity {
                        address: self.address.clone(),
                        domain: EntityDomain::MediaPlayer.to_string(),
                    })?;

                self.send(MEDIA_PLAYER_COMMAND_REQUEST, &media_command(key, &command)?)
                    .await
            }
        }
    }

    pub async fn close(&mut self) {
        let _ = self
            .send(
                DISCONNECT_REQUEST,
                &proto::DisconnectRequest {
                    reason: proto::DisconnectReason::Unspecified.into(),
                },
            )
            .await;

        let _ = self.writer.shutdown().await;
    }
}

fn with_port(address: &str, port: u16) -> String {
    if address.rsplit_once(':').is_some_and(|(_, port)| {
        !port.is_empty() && port.chars().all(|character| character.is_ascii_digit())
    }) {
        return address.to_owned();
    }

    format!("{address}:{port}")
}

fn byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn light_state(state: &proto::LightStateResponse) -> Value {
    json!({
        "state": if state.state { "ON" } else { "OFF" },
        "brightness": byte(state.brightness),
        "color": {
            "hex": format!(
                "#{:02x}{:02x}{:02x}",
                byte(state.red),
                byte(state.green),
                byte(state.blue),
            ),
        },
    })
}

fn fan_state(state: &proto::FanStateResponse) -> Value {
    json!({
        "state": if state.state { "ON" } else { "OFF" },
        "speed": state.speed_level,
        "preset": (!state.preset_mode.is_empty()).then_some(&state.preset_mode),
    })
}

const FAN_FIELDS: [&str; 3] = ["state", "speed", "preset"];

fn switch_commands(payload: &Value) -> Vec<(String, bool)> {
    let Some(switches) = payload.get("switches").and_then(Value::as_object) else {
        return Vec::new();
    };

    switches
        .iter()
        .filter_map(|(object_id, state)| Some((object_id.clone(), state.as_bool()?)))
        .collect()
}

fn fan_command(key: u32, payload: &Value) -> proto::FanCommandRequest {
    let mut request = proto::FanCommandRequest {
        key,
        ..Default::default()
    };

    if let Some(state) = payload.get("state").and_then(Value::as_str) {
        request.has_state = true;
        request.state = state.eq_ignore_ascii_case("on");
    }

    if let Some(speed) = payload.get("speed").and_then(Value::as_i64) {
        request.has_speed_level = true;
        request.speed_level = speed as i32;
    }

    if let Some(preset) = payload.get("preset").and_then(Value::as_str) {
        request.has_preset_mode = true;
        request.preset_mode = preset.to_owned();
    }

    request
}

fn media_player_state(state: &proto::MediaPlayerStateResponse) -> Value {
    let name = match proto::MediaPlayerState::try_from(state.state) {
        Ok(proto::MediaPlayerState::Idle) => "idle",
        Ok(proto::MediaPlayerState::Playing) => "playing",
        Ok(proto::MediaPlayerState::Paused) => "paused",
        Ok(proto::MediaPlayerState::Announcing) => "announcing",
        Ok(proto::MediaPlayerState::Off) => "off",
        Ok(proto::MediaPlayerState::On) => "on",
        Ok(proto::MediaPlayerState::None) | Err(_) => "unknown",
    };

    json!({
        "state": name,
        "volume": f64::from(state.volume),
        "muted": state.muted,
    })
}

fn channel(payload: &Value, key: &str) -> Option<f32> {
    let channel = payload.get("color")?.get(key)?.as_f64()?;

    Some((channel / 255.0) as f32)
}

fn light_command(key: u32, payload: &Value) -> proto::LightCommandRequest {
    let mut request = proto::LightCommandRequest {
        key,
        ..Default::default()
    };

    if let Some(state) = payload.get("state").and_then(Value::as_str) {
        request.has_state = true;
        request.state = state.eq_ignore_ascii_case("on");
    }

    if let Some(brightness) = payload.get("brightness").and_then(Value::as_f64) {
        request.has_brightness = true;
        request.brightness = (brightness.clamp(0.0, 255.0) / 255.0) as f32;
    }

    if let (Some(red), Some(green), Some(blue)) = (
        channel(payload, "r"),
        channel(payload, "g"),
        channel(payload, "b"),
    ) {
        request.has_rgb = true;
        request.red = red;
        request.green = green;
        request.blue = blue;
    }

    request
}

fn media_command(
    key: u32,
    command: &MediaCommand,
) -> Result<proto::MediaPlayerCommandRequest, EsphomeNativeApiError> {
    let mut request = proto::MediaPlayerCommandRequest {
        key,
        ..Default::default()
    };

    let command = match command {
        MediaCommand::Play => proto::MediaPlayerCommand::Play,
        MediaCommand::Pause => proto::MediaPlayerCommand::Pause,
        MediaCommand::PlayPause => proto::MediaPlayerCommand::Toggle,
        MediaCommand::Stop => proto::MediaPlayerCommand::Stop,
        MediaCommand::Next | MediaCommand::Previous => {
            return Err(EsphomeNativeApiError::Unsupported(command.to_string()));
        }
        MediaCommand::Mute(true) => proto::MediaPlayerCommand::Mute,
        MediaCommand::Mute(false) => proto::MediaPlayerCommand::Unmute,
        MediaCommand::TurnOff => proto::MediaPlayerCommand::TurnOff,
        MediaCommand::Volume(volume) => {
            request.has_volume = true;
            request.volume = volume.clamp(0.0, 1.0) as f32;

            return Ok(request);
        }
        MediaCommand::PlayMedia { url, announcement } => {
            request.has_media_url = true;
            request.media_url = url.clone();
            request.has_announcement = true;
            request.announcement = *announcement;

            return Ok(request);
        }
    };

    request.has_command = true;
    request.command = command.into();

    Ok(request)
}

trait Message: prost::Message + Sized {}
impl<M: prost::Message + Sized> Message for M {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_keeps_its_own_port() {
        assert_eq!(
            with_port("apollo-cast-1-livingroom.iot", 6053),
            "apollo-cast-1-livingroom.iot:6053"
        );
        assert_eq!(with_port("10.0.2.15:6054", 6053), "10.0.2.15:6054");
    }

    #[test]
    fn a_light_state_reads_like_the_mqtt_one() {
        let state = proto::LightStateResponse {
            state: true,
            brightness: 1.0,
            red: 1.0,
            green: 0.0,
            blue: 0.0627451,
            ..Default::default()
        };

        assert_eq!(
            light_state(&state),
            json!({ "state": "ON", "brightness": 255, "color": { "hex": "#ff0010" } })
        );
    }

    #[test]
    fn a_light_command_scales_to_the_native_floats() {
        let request = light_command(
            7,
            &json!({ "state": "ON", "brightness": 255, "color": { "r": 255, "g": 0, "b": 16 } }),
        );

        assert_eq!(request.key, 7);
        assert!(request.has_state && request.state);
        assert!(request.has_brightness);
        assert_eq!(request.brightness, 1.0);
        assert!(request.has_rgb);
        assert_eq!(request.red, 1.0);
        assert_eq!(request.green, 0.0);
    }

    #[test]
    fn a_fan_state_carries_its_speed_and_preset() {
        let state = proto::FanStateResponse {
            state: true,
            speed_level: 2,
            preset_mode: "Manual".to_owned(),
            ..Default::default()
        };

        assert_eq!(
            fan_state(&state),
            json!({ "state": "ON", "speed": 2, "preset": "Manual" })
        );

        assert_eq!(
            fan_state(&proto::FanStateResponse::default()),
            json!({ "state": "OFF", "speed": 0, "preset": null })
        );
    }

    #[test]
    fn a_fan_command_only_sets_what_it_was_given() {
        let request = fan_command(9, &json!({ "state": "ON", "preset": "Auto" }));

        assert_eq!(request.key, 9);
        assert!(request.has_state && request.state);
        assert!(request.has_preset_mode);
        assert_eq!(request.preset_mode, "Auto");
        assert!(!request.has_speed_level);

        let request = fan_command(9, &json!({ "speed": 3 }));

        assert!(!request.has_state);
        assert!(request.has_speed_level);
        assert_eq!(request.speed_level, 3);
    }

    #[test]
    fn a_fan_payload_can_carry_switch_commands() {
        assert_eq!(
            switch_commands(&json!({ "switches": { "display": false } })),
            vec![("display".to_owned(), false)]
        );
        assert!(switch_commands(&json!({ "state": "ON" })).is_empty());
    }

    #[test]
    fn play_media_carries_its_url() {
        let request = media_command(
            3,
            &MediaCommand::PlayMedia {
                url: "http://media/track.flac".to_owned(),
                announcement: false,
            },
        )
        .expect("command");

        assert!(request.has_media_url);
        assert_eq!(request.media_url, "http://media/track.flac");
        assert!(!request.has_command);
    }

    #[test]
    fn track_skipping_is_not_supported_by_the_api() {
        assert!(media_command(3, &MediaCommand::Next).is_err());
    }

    #[test]
    fn a_media_player_state_lowercases_its_name() {
        let state = proto::MediaPlayerStateResponse {
            state: proto::MediaPlayerState::Playing.into(),
            volume: 0.5,
            muted: false,
            ..Default::default()
        };

        assert_eq!(
            media_player_state(&state),
            json!({ "state": "playing", "volume": 0.5, "muted": false })
        );
    }
}
