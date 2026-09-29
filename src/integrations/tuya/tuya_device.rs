use tokio::sync::mpsc;

use super::command::Command;

pub struct TuyaDevice {
    pub address: String,
    pub host: String,
    pub(super) local_key: [u8; 16],
    pub(super) receiver: mpsc::Receiver<Command>,
}
