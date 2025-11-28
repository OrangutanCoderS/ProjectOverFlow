use thiserror::Error;
use chrono::ParseError as ChronoParseError;
use ed25519_dalek::SignatureError;
use base64::DecodeError as Base64DecodeError;

/// Unified error type for the authentication layer.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("unknown peer id: {0}")]
    UnknownPeer(String),

    #[error("base64 decode error: {0}")]
    Base64(#[from] Base64DecodeError),

    #[error("invalid public key length: {0}")]
    BadPublicKeyLen(usize),

    #[error("invalid signature length: {0}")]
    BadSignatureLen(usize),

    #[error("signature verification failed: {0}")]
    Signature(#[from] SignatureError),

    #[error("timestamp parse error: {0}")]
    Timestamp(#[from] ChronoParseError),

    #[error("clock skew too large")]
    ClockSkew,

    #[error("replay detected")]
    Replay,
}

pub type AuthResult<T> = Result<T, AuthError>;
