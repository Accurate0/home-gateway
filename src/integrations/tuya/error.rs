#[derive(thiserror::Error, Debug)]
pub enum TuyaError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("connecting timed out after {0:?}")]
    ConnectTimeout(std::time::Duration),
    #[error("failed to encrypt a frame")]
    Encrypt,
    #[error("a frame failed authentication, the key is wrong or the stream is out of sync")]
    Authentication,
    #[error("unexpected frame prefix {0:#010x}")]
    InvalidPrefix(u32),
    #[error("unexpected frame suffix {0:#010x}")]
    InvalidSuffix(u32),
    #[error("frame length {0} is out of range")]
    InvalidLength(usize),
    #[error("session key negotiation failed: {0}")]
    Negotiation(String),
    #[error("nothing heard for {0:?}, assuming the connection is dead")]
    Silent(std::time::Duration),
    #[error("the device closed the connection")]
    Closed,
    #[error("{address} is not connected")]
    NotConnected { address: String },
    #[error("{address}: tuya commands are a json object of data points")]
    InvalidDps { address: String },
}
