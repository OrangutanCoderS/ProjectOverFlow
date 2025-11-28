use crate::error::TelemetryError;
use crate::model::TelemetryEnvelope;
use async_trait::async_trait;
use std::sync::Arc;

/// Abstraction for something that can send a telemetry envelope.
///
/// This is how we decouple from actual network/auth logic.
#[async_trait]
pub trait TelemetrySink: Send + Sync {
    /// Send a single envelope. Implementations should be fast or internally
    /// offload heavy work.
    async fn send(&self, env: TelemetryEnvelope) -> Result<(), TelemetryError>;
}

/// Simple in-memory sink for tests and benches. Just stores envelopes.
#[derive(Debug, Default)]
pub struct InMemorySink {
    inner: Arc<parking_lot::Mutex<Vec<TelemetryEnvelope>>>,
}

impl InMemorySink {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.lock().len()
    }

    pub fn envelopes(&self) -> Vec<TelemetryEnvelope> {
        self.inner.lock().clone()
    }
}

#[async_trait]
impl TelemetrySink for InMemorySink {
    async fn send(&self, env: TelemetryEnvelope) -> Result<(), TelemetryError> {
        self.inner.lock().push(env);
        Ok(())
    }
}
