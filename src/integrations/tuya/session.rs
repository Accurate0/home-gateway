use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc;

use crate::settings::TuyaSettings;

use super::command::Command;
use super::command_code::{
    CONTROL, CONTROL_NEW, DP_QUERY, DP_QUERY_NEW, HEART_BEAT, SESS_KEY_NEG_FINISH,
    SESS_KEY_NEG_RESP, SESS_KEY_NEG_START, STATUS,
};
use super::dps_update::DpsUpdate;
use super::error::TuyaError;
use super::frame::{self, Frame};
use super::negotiation;
use super::payload;
use super::tuya_device::TuyaDevice;

const READ_CHUNK: usize = 4096;

pub struct Session {
    address: String,
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
    buffer: Vec<u8>,
    key: [u8; 16],
    seq: u32,
    last_heard: Instant,
}

impl Session {
    pub async fn connect(device: &TuyaDevice, settings: &TuyaSettings) -> Result<Self, TuyaError> {
        let target = format!("{}:{}", device.host, settings.port);
        let timeout = settings.silence_timeout();

        tracing::debug!("connecting to tuya device {} at {target}", device.address);

        let stream = tokio::time::timeout(timeout, TcpStream::connect(&target))
            .await
            .map_err(|_| TuyaError::ConnectTimeout(timeout))??;

        stream.set_nodelay(true)?;

        let (reader, writer) = stream.into_split();

        let mut session = Session {
            address: device.address.clone(),
            reader,
            writer,
            buffer: Vec::new(),
            key: device.local_key,
            seq: 0,
            last_heard: Instant::now(),
        };

        session.negotiate(&device.local_key, timeout).await?;

        tracing::debug!(
            "negotiated a session key with tuya device {}",
            device.address
        );

        Ok(session)
    }

    async fn negotiate(
        &mut self,
        local_key: &[u8; 16],
        timeout: Duration,
    ) -> Result<(), TuyaError> {
        let local_nonce: [u8; 16] = rand::random();

        self.send(SESS_KEY_NEG_START, &local_nonce).await?;

        let response = tokio::time::timeout(timeout, self.receive(SESS_KEY_NEG_RESP))
            .await
            .map_err(|_| TuyaError::Silent(timeout))??;

        let remote_nonce = negotiation::remote_nonce(local_key, &local_nonce, &response.payload)?;

        self.send(
            SESS_KEY_NEG_FINISH,
            &negotiation::hmac(local_key, &remote_nonce),
        )
        .await?;

        self.key = negotiation::session_key(local_key, &local_nonce, &remote_nonce)?;

        Ok(())
    }

    async fn send(&mut self, cmd: u32, payload: &[u8]) -> Result<(), TuyaError> {
        self.seq = self.seq.wrapping_add(1);

        let frame = Frame {
            seq: self.seq,
            cmd,
            payload: payload.to_vec(),
        };

        let bytes = frame::encode(&self.key, &frame)?;

        self.writer.write_all(&bytes).await?;

        Ok(())
    }

    async fn read_more(&mut self) -> Result<(), TuyaError> {
        let mut chunk = [0; READ_CHUNK];

        let read = self.reader.read(&mut chunk).await?;

        self.received(&chunk[..read])
    }

    fn received(&mut self, bytes: &[u8]) -> Result<(), TuyaError> {
        if bytes.is_empty() {
            return Err(TuyaError::Closed);
        }

        self.buffer.extend_from_slice(bytes);
        self.last_heard = Instant::now();

        Ok(())
    }

    async fn receive(&mut self, cmd: u32) -> Result<Frame, TuyaError> {
        loop {
            while let Some(frame) = frame::take(&mut self.buffer, &self.key)? {
                if frame.cmd == cmd {
                    return Ok(frame);
                }

                tracing::debug!(
                    "tuya device {} sent command {:#04x} while waiting for {cmd:#04x}",
                    self.address,
                    frame.cmd
                );
            }

            self.read_more().await?;
        }
    }

    pub async fn run(
        &mut self,
        commands: &mut mpsc::Receiver<Command>,
        settings: &TuyaSettings,
        mut on_dps: impl FnMut(DpsUpdate),
    ) -> Result<(), TuyaError> {
        let silence_timeout = settings.silence_timeout();

        let mut heartbeat = tokio::time::interval(settings.heartbeat());
        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        heartbeat.tick().await;

        self.send(DP_QUERY_NEW, b"{}").await?;

        let mut chunk = [0; READ_CHUNK];

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    if self.last_heard.elapsed() > silence_timeout {
                        return Err(TuyaError::Silent(silence_timeout));
                    }

                    let beat = json!({ "gwId": self.address, "devId": self.address });

                    self.send(HEART_BEAT, &serde_json::to_vec(&beat)?).await?;
                }
                command = commands.recv() => {
                    match command {
                        Some(command) => self.command(command).await?,
                        None => return Ok(()),
                    }
                }
                read = self.reader.read(&mut chunk) => {
                    self.received(&chunk[..read?])?;

                    while let Some(frame) = frame::take(&mut self.buffer, &self.key)? {
                        self.handle(frame, &mut on_dps)?;
                    }
                }
            }
        }
    }

    fn handle(&self, frame: Frame, on_dps: &mut impl FnMut(DpsUpdate)) -> Result<(), TuyaError> {
        match frame.cmd {
            HEART_BEAT => {
                tracing::trace!("tuya device {} answered a heartbeat", self.address);
            }
            STATUS | DP_QUERY | DP_QUERY_NEW | CONTROL | CONTROL_NEW => {
                match payload::dps(&frame.payload)? {
                    Some(dps) => {
                        tracing::debug!("tuya device {} reported {dps:?}", self.address);

                        on_dps(DpsUpdate {
                            address: self.address.clone(),
                            dps,
                        });
                    }
                    None => tracing::trace!(
                        "tuya device {} acknowledged command {:#04x}",
                        self.address,
                        frame.cmd
                    ),
                }
            }
            other => {
                tracing::debug!(
                    "ignoring command {other:#04x} from tuya device {}",
                    self.address
                );
            }
        }

        Ok(())
    }

    async fn command(&mut self, command: Command) -> Result<(), TuyaError> {
        match command {
            Command::SetDps(dps) => {
                tracing::info!("setting {dps:?} on tuya device {}", self.address);

                self.send(CONTROL_NEW, &control_payload(dps)?).await
            }
        }
    }

    pub async fn close(&mut self) {
        let _ = self.writer.shutdown().await;
    }
}

fn control_payload(dps: Map<String, Value>) -> Result<Vec<u8>, TuyaError> {
    let body = json!({
        "protocol": 5,
        "t": chrono::Utc::now().timestamp(),
        "data": { "dps": dps },
    });

    let mut payload = payload::version_header().to_vec();
    payload.extend(serde_json::to_vec(&body)?);

    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_control_payload_carries_the_version_header_and_nested_dps() {
        let dps = json!({ "101": "fopen" })
            .as_object()
            .cloned()
            .expect("object");

        let payload = control_payload(dps).expect("payload");

        assert!(payload.starts_with(b"3.5\0\0\0\0\0\0\0\0\0\0\0\0{"));

        let body: Value = serde_json::from_slice(payload::json_body(&payload)).expect("json");

        assert_eq!(body["protocol"], 5);
        assert_eq!(body["data"]["dps"]["101"], "fopen");
    }
}
