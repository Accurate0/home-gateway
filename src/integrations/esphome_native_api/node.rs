use tokio::sync::mpsc;

use super::command::Command;

pub struct Node {
    pub address: String,
    pub(super) receiver: mpsc::Receiver<Command>,
}
