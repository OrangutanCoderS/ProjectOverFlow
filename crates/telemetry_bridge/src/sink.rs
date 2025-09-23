use crate::error::TelemetryError;
use crate::packet::TelemetryPacket;
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// A sink consumes telemetry packets.
/// Additional sinks (SocketSink, IpcSink) can be implemented later.
pub trait TelemetrySink: Send + Sync {
    fn publish(&self, packet: &TelemetryPacket) -> Result<(), TelemetryError>;
}

/// A simple JSONL file sink.
/// One packet per line, serialized compactly (no pretty printing).
pub struct FileSink {
    path: PathBuf,
    writer: Mutex<BufWriter<File>>,
}

impl FileSink {
    pub fn new(path: PathBuf) -> Result<Self, TelemetryError> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            path,
            writer: Mutex::new(BufWriter::new(file)),
        })
    }

    /// For testing/introspection
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl TelemetrySink for FileSink {
    fn publish(&self, packet: &TelemetryPacket) -> Result<(), TelemetryError> {
        let mut w = self.writer.lock();
        let line = serde_json::to_string(packet)?; // compact
        w.write_all(line.as_bytes())?;
        w.write_all(b"\n")?;
        w.flush()?; // durability; change to buffered if you need speed
        Ok(())
    }
}
