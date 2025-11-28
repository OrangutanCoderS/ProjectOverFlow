use crate::error::TelemetryError;
use crate::model::{ForwardResult, TelemetryCategory, TelemetryEnvelope, TelemetryPayload};
use crate::sink::TelemetrySink;
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;
use tracing::{debug, error};

/// Internal command for the worker.
#[derive(Debug)]
enum QueueCmd {
    Enqueue(TelemetryPayload),
    Flush,
}

/// TelemetryForwarder: small, focused outbound pipe.
///
/// - Builds canonical envelopes from raw payloads.
/// - Either sends immediately or pushes into a bounded queue.
/// - Uses an abstract TelemetrySink to actually deliver envelopes.
pub struct TelemetryForwarder<S: TelemetrySink + 'static> {
    origin_id: String,
    sink: Arc<S>,
    queue_tx: mpsc::Sender<QueueCmd>,
}

impl<S: TelemetrySink + 'static> TelemetryForwarder<S> {
    /// Create a new forwarder with bounded queue.
    ///
    /// `origin_id`    - logical id of this node.
    /// `sink`         - implementation of TelemetrySink (network/auth/etc.).
    /// `queue_capacity` - maximum queued events before we start rejecting.
    pub fn new(origin_id: String, sink: Arc<S>, queue_capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(queue_capacity.max(1));
        let worker_sink = Arc::clone(&sink);
        let worker_origin = origin_id.clone();

        // Spawn background worker.
        task::spawn(async move {
            worker_loop(worker_origin, worker_sink, rx).await;
        });

        Self {
            origin_id,
            sink,
            queue_tx: tx,
        }
    }

    /// Build a canonical envelope from payload plus optional trace_id and auth.
    fn build_envelope(
        &self,
        payload: TelemetryPayload,
        trace_id: Option<String>,
        auth: Option<Value>,
    ) -> TelemetryEnvelope {
        TelemetryEnvelope {
            origin_id: self.origin_id.clone(),
            timestamp: Utc::now(),
            trace_id,
            category: payload.category,
            body: payload.body,
            auth,
        }
    }

    /// Immediately send a single telemetry event through the sink.
    ///
    /// This bypasses the queue and is suitable for critical alerts.
    pub async fn send_telemetry_now(
        &self,
        payload: TelemetryPayload,
        trace_id: Option<String>,
        auth: Option<Value>,
    ) -> Result<ForwardResult, TelemetryError> {
        let env = self.build_envelope(payload, trace_id, auth);
        self.sink.send(env).await.map_err(|e| {
            TelemetryError::SinkFailure(format!("immediate send failed: {e}"))
        })?;

        Ok(ForwardResult::accepted(None))
    }

    /// Enqueue telemetry for background sending.
    ///
    /// If the queue is full, returns `TelemetryError::QueueFull`.
    pub async fn queue_telemetry(
        &self,
        payload: TelemetryPayload,
    ) -> Result<ForwardResult, TelemetryError> {
        self.queue_tx
            .try_send(QueueCmd::Enqueue(payload))
            .map_err(|_e| TelemetryError::QueueFull)?;

        Ok(ForwardResult::accepted(Some(
            "queued for background forwarding".to_string(),
        )))
    }

    /// Enqueue a flush command (best-effort).
    pub async fn flush(&self) {
        if let Err(e) = self.queue_tx.try_send(QueueCmd::Flush) {
            debug!("telemetry_forwarder: flush command dropped: {e:?}");
        }
    }

    /// Convenience sugar for common categories to keep daemon code clean.
    pub async fn queue_heartbeat(&self, body: Value) -> Result<ForwardResult, TelemetryError> {
        let payload = TelemetryPayload {
            category: TelemetryCategory::Heartbeat,
            body,
        };
        self.queue_telemetry(payload).await
    }
}

/// Background worker that drains the queue and calls the sink.
async fn worker_loop<S: TelemetrySink + 'static>(
    origin_id: String,
    sink: Arc<S>,
    mut rx: mpsc::Receiver<QueueCmd>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            QueueCmd::Enqueue(payload) => {
                let env = TelemetryEnvelope {
                    origin_id: origin_id.clone(),
                    timestamp: Utc::now(),
                    trace_id: None,
                    category: payload.category,
                    body: payload.body,
                    auth: None,
                };

                if let Err(e) = sink.send(env).await {
                    error!("telemetry_forwarder: sink error: {e}");
                }
            }
            QueueCmd::Flush => {
                debug!("telemetry_forwarder: flush requested (no-op placeholder)");
                // later: implement drain-with-confirmation if needed
            }
        }
    }
}
