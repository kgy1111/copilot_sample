use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("encode error: {0}")]
    EncodeError(String),

    #[error("decode error: {0}")]
    DecodeError(String),

    #[error("connect timeout")]
    ConnectTimeout,

    #[error("send/receive timeout")]
    Timeout,

    #[error("close timeout, forced disconnect")]
    CloseTimeout,

    #[error("not connected")]
    NotConnected,

    #[error("USB device not found")]
    DeviceNotFound,

    #[error("USB endpoint not found")]
    EndpointNotFound,

    #[error("transport error: {0}")]
    Transport(Box<dyn std::error::Error + Send + Sync + 'static>),
}
