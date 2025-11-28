use thiserror::Error;

#[derive(Debug, Error)]
pub enum PeerSyncError {
    #[error("invalid packet format")]
    InvalidPacket,

    #[error("signature verification failed")]
    BadSignature,

    #[error("network IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    SerdeErr(#[from] serde_json::Error),
}
