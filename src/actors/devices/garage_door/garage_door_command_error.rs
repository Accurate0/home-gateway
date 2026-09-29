use crate::actors::system::rpc::RpcError;

#[derive(Debug, thiserror::Error)]
pub enum GarageDoorCommandError {
    #[error(transparent)]
    Rpc(#[from] RpcError),
    #[error("{0}")]
    Rejected(String),
}
