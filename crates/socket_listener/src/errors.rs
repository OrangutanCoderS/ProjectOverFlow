use thiserror::Error;

/// Error type for the socket listener crate.
/// All failures in this crate should map into one of these variants.
#[derive(Debug, Error)]
pub enum ListenerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TLS handshake failed: {0}")]
    TlsHandshake(String),

    #[error("Invalid packet format: {0}")]
    InvalidPacket(String),

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Replay attack detected")]
    ReplayAttack,

    #[error("Rate limit exceeded for peer")]
    RateLimited,

    #[error("Timeout while reading from socket")]
    Timeout,

    #[error("Router error: {0}")]
    Router(String),
}
