//! Module 26 — Telemetry Bridge
//! A small façade to collect telemetry packets from modules and fan them out
//! to one or more sinks (file, socket, IPC in the future).
//!
//! Design goals:
//! - Zero-cost abstraction on happy-path (simple write path)
//! - Clear error surface
//! - Minimal locking (only around shared sink list & cache)
//! - Optional in-memory ring buffer for the last N packets

mod error;
mod packet;
mod sink;
mod bridge;

pub use crate::error::TelemetryError;
pub use crate::packet::TelemetryPacket;
pub use crate::sink::{TelemetrySink, FileSink};
pub use crate::bridge::{TelemetryBridge, BridgeConfig};
