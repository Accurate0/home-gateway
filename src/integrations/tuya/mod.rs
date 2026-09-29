mod command;
mod command_code;
mod dps_update;
mod error;
mod frame;
mod negotiation;
mod payload;
mod session;
mod tuya_device;

use std::collections::HashMap;

use ractor::{
    ActorRef,
    factory::{FactoryMessage, Job, JobOptions},
};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::actors::system::tuya_ingest::{self, TuyaIngest};
use crate::device_registry::DeviceRegistry;
use crate::reconnect::{Reachability, Reconnect};
use crate::settings::TuyaSettings;

use command::Command;
use session::Session;

pub use dps_update::DpsUpdate;
pub use error::TuyaError;
pub use tuya_device::TuyaDevice;

const COMMAND_QUEUE: usize = 16;

#[derive(Clone)]
pub struct Tuya {
    devices: HashMap<String, mpsc::Sender<Command>>,
}

impl Tuya {
    pub fn new(registry: &DeviceRegistry, settings: &TuyaSettings) -> (Self, Vec<TuyaDevice>) {
        let mut senders = HashMap::new();
        let mut devices = Vec::new();

        for device in registry.tuya_devices() {
            let Some(config) = settings.devices.get(&device.id) else {
                tracing::error!(
                    "tuya device {} has no connection settings, skipping it",
                    device.id
                );
                continue;
            };

            let local_key = config
                .local_key
                .as_deref()
                .and_then(|key| <[u8; 16]>::try_from(key.as_bytes()).ok());

            let Some(local_key) = local_key else {
                tracing::error!(
                    "tuya device {} has no usable local key, skipping it",
                    device.id
                );
                continue;
            };

            let (sender, receiver) = mpsc::channel(COMMAND_QUEUE);

            senders.insert(device.address.clone(), sender);
            devices.push(TuyaDevice {
                address: device.address.clone(),
                host: config.host.clone(),
                local_key,
                receiver,
            });
        }

        (Self { devices: senders }, devices)
    }

    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    pub async fn set_dps(&self, address: &str, dps: Value) -> Result<(), TuyaError> {
        let Value::Object(dps) = dps else {
            return Err(TuyaError::InvalidDps {
                address: address.to_owned(),
            });
        };

        let Some(device) = self.devices.get(address) else {
            return Err(TuyaError::NotConnected {
                address: address.to_owned(),
            });
        };

        device
            .send(Command::SetDps(dps))
            .await
            .map_err(|_| TuyaError::NotConnected {
                address: address.to_owned(),
            })
    }
}

fn ingest_actor() -> Option<ActorRef<FactoryMessage<String, tuya_ingest::Message>>> {
    ractor::registry::where_is(TuyaIngest::NAME).map(ActorRef::from)
}

fn dispatch(update: DpsUpdate) {
    let address = update.address.clone();

    send(address, tuya_ingest::Message::Dps(update));
}

fn dispatch_connection(reachability: &mut Reachability, address: &str, connected: bool) {
    if !reachability.changed(connected) {
        return;
    }

    send(
        address.to_owned(),
        tuya_ingest::Message::Connection {
            address: address.to_owned(),
            connected,
        },
    );
}

fn send(address: String, msg: tuya_ingest::Message) {
    let Some(actor) = ingest_actor() else {
        tracing::error!("tuya ingest actor is not registered, dropping {address}");
        return;
    };

    let response = actor.send_message(FactoryMessage::Dispatch(Job {
        key: address,
        msg,
        options: JobOptions::default(),
        accepted: None,
    }));

    if let Err(e) = response {
        tracing::error!("error sending to tuya ingest actor: {e}");
    }
}

pub async fn process_events(
    mut device: TuyaDevice,
    settings: TuyaSettings,
    cancellation_token: CancellationToken,
) {
    let mut reachability = Reachability::default();
    let mut reconnect = Reconnect::new(settings.reconnect);

    loop {
        tokio::select! {
            result = connected(&mut device, &settings, &mut reachability, &mut reconnect) => {
                let attempt = reconnect.failed();

                if let Err(e) = result {
                    attempt.log_failure(format_args!("tuya device {}", device.address), e);
                }

                dispatch_connection(&mut reachability, &device.address, false);

                tokio::time::sleep(attempt.delay).await;
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("tuya device {} cancellation requested", device.address);
                return;
            }
        }
    }
}

async fn connected(
    device: &mut TuyaDevice,
    settings: &TuyaSettings,
    reachability: &mut Reachability,
    reconnect: &mut Reconnect,
) -> Result<(), TuyaError> {
    let mut session = Session::connect(device, settings).await?;

    if reconnect.connected() {
        tracing::info!("tuya device {} reconnected", device.address);
    }

    dispatch_connection(reachability, &device.address, true);

    let result = session.run(&mut device.receiver, settings, dispatch).await;

    session.close().await;

    result
}
