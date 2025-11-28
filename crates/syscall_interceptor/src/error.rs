use thiserror::Error;

/// Errors that syscall_interceptor may return.
#[derive(Debug, Error)]
pub enum InterceptorError {
    #[error("failed to attach: {0}")]
    AttachFailed(String),

    #[error("failed to detach: {0}")]
    DetachFailed(String),

    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    #[error("channel closed unexpectedly")]
    ChannelClosed,
}