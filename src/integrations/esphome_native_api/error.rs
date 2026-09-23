#[derive(thiserror::Error, Debug)]
pub enum EsphomeNativeApiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Noise(#[from] snow::Error),
    #[error(transparent)]
    Decode(#[from] prost::DecodeError),
    #[error("encryption key is not a base64 encoded 32 byte value")]
    InvalidEncryptionKey,
    #[error("the device is speaking plaintext, but the gateway is configured with a key")]
    Plaintext,
    #[error("unexpected frame marker {0:#04x}")]
    InvalidMarker(u8),
    #[error("the device rejected the handshake: {0}")]
    Handshake(String),
    #[error("the device announced protocol {0}, expected 1")]
    UnknownProtocol(u8),
    #[error("frame is too short: {0} bytes")]
    ShortFrame(usize),
    #[error("no message for {0:?}, assuming the connection is dead")]
    Silent(std::time::Duration),
    #[error("the device closed the connection: {0}")]
    Disconnected(String),
    #[error("{address} is not connected")]
    NotConnected { address: String },
    #[error("{address} has no {domain} entity to command")]
    NoEntity { address: String, domain: String },
    #[error("{0} does not support this command")]
    Unsupported(String),
}
